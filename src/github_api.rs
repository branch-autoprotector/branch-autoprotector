/// Configuration of the GitHub API client.
#[derive(serde::Deserialize)]
pub struct Config
{
	/// The base URL of the GitHub API server with a trailing slash (optional, default:
	/// <https://api.github.com/>).
	#[serde(default = "github_com_api_base_url")]
	base_url: url::Url,
	/// The slug of the organization this service watches, as included in URLs (for an organization
	/// with the URL <https://github.com/example-organization>, this would be
	/// `example-organization`).
	organization: String,
	/// Path to the private key that was generated for the GitHub App. Make sure to set the
	/// permissions in such a way that other users on this machine can’t read it.
	private_key_path: std::path::PathBuf,
	/// The numeric App ID of this GitHub App as shown at the top of its *About* page.
	app_id: u64,
	/// To verify that incoming webhook payloads actually come from GitHub.com, provide the GitHub
	/// App’s webhook secret (optional, but recommended for production use).
	webhook_secret: Option<String>,
}

#[doc(hidden)]
fn github_com_api_base_url() -> url::Url
{
	url::Url::parse("https://api.github.com/")
		.expect("this call is infallible because we know the URL to be well-formed")
}

/// A GitHub API client that authenticates with a GitHub server as a GitHub App.
///
/// The GitHub API automatically authenticates using an installation of a specific organization,
/// allowing you to make GitHub API calls to resources in that organization, provided that the
/// GitHub App has the proper permissions configured. This is achieved by automatically obtaining an
/// installation access token. The client automatically renews the token once it expires. Also, the
/// client retries API requests that failed for reasons such as network issues multiple times for a
/// total of up to five minutes.
///
/// Currently, the GitHub API client supports only a single organization.
///
/// The client can safely be shared between threads, which is achieved by internally using
/// thread-safe handles to the underlying data structures. This allows the client to be used in
/// request handlers asynchronously and concurrently.
#[derive(Clone)]
pub struct Client
{
	#[doc(hidden)]
	config: std::sync::Arc<Config>,
	#[doc(hidden)]
	reqwest_client: reqwest_middleware::ClientWithMiddleware,
	#[doc(hidden)]
	private_key: jsonwebtoken::EncodingKey,
	#[doc(hidden)]
	// The access token is protected by a read–write lock. In this way, tasks can read the token
	// without blocking each other. In the rare event that the token expired, it can be locked for
	// writing in order to refresh it. Thanks to tokio’s implementation of read–write locks, writers
	// take precedence over readers. This avoids starvations issues where many tasks recognize the
	// access token as expired but none of them succeed in acquiring the write lock that would be
	// necessary to refresh the access token because there are still more readers waiting
	access_token: std::sync::Arc<tokio::sync::RwLock<AccessToken>>,
}

impl Client
{
	/// Initialize a new GitHub API client with a given configuration.
	pub async fn from_config(config: Config) -> Result<Self, crate::Error>
	{
		let config = std::sync::Arc::new(config);

		// Read and parse the GitHub App’s private key from the .pem file
		let private_key = std::fs::read(&config.private_key_path)
			.map_err(crate::Error::ReadPrivateGitHubAppKeyFile)?;
		let private_key = jsonwebtoken::EncodingKey::from_rsa_pem(&private_key)
			.map_err(crate::Error::ParsePrivateGitHubAppKeyFile)?;

		// Initialize a new HTTP client
		let reqwest_client = reqwest::ClientBuilder::new()
			// Set a recognizable user agent to get meaningful debugging information from GitHub
			.user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
			.build().map_err(crate::Error::CreateHttpClient)?;

		// Wrap the HTTP client in middleware that retries requests for up to 5 minutes in case of
		// network failures
		let retry_policy = reqwest_retry::policies::ExponentialBackoff::builder()
			.backoff_exponent(2)
			.retry_bounds(std::time::Duration::from_secs(1), std::time::Duration::from_secs(60))
			.build_with_total_retry_duration(std::time::Duration::from_secs(5 * 60));
		let retry_transient_middleware =
			reqwest_retry::RetryTransientMiddleware::new_with_policy(retry_policy);

		let reqwest_client = reqwest_middleware::ClientBuilder::new(reqwest_client)
			.with(retry_transient_middleware)
			.build();

		// Request an initial access token from GitHub for this GitHub App and the organization it’s
		// installed to
		log::info!("requesting GitHub App installation access token");
		let access_token = AccessToken::new(&config, &private_key, &reqwest_client).await?;
		let access_token = std::sync::Arc::new(tokio::sync::RwLock::new(access_token));

		Ok(Self
		{
			config,
			reqwest_client,
			private_key,
			access_token,
		})
	}

	/// Make an HTTP request to the GitHub API.
	///
	/// # Arguments
	/// - `method`: The HTTP method to use (example: [reqwest::Method::POST]).
	/// - `endpoint`: The API endpoint (without host and leading slash, example:
	///   `repos/example_organization`).
	/// - `body`: A serializable type containing the request body.
	pub async fn request<S, B, R>(&self, method: reqwest::Method, endpoint: S, body: Option<&B>)
		-> Result<R, crate::Error>
	where
		S: AsRef<str>,
		B: serde::Serialize,
		R: serde::de::DeserializeOwned,
	{
		let endpoint = endpoint.as_ref();

		// Copy the access token by value, as we might need to check whether its value changed if we
		// need to make a second attempt because of an expired access token
		let mut access_token = (*self.access_token.read().await).clone();

		// Try making the GitHub API request with the provided access token
		match request(&self.config, &self.reqwest_client, method.clone(), endpoint, body,
			&access_token).await
		{
			// If the request failed with a 401 Unauthorized status code, check if the access token
			// has expired and retry with a fresh one
			Err(crate::Error::ReceivedGitHubApiClientError{status_code, ..})
				if status_code == reqwest::StatusCode::UNAUTHORIZED =>
			{
				{
					let mut access_token_locked = self.access_token.write().await;

					// The access token might already have been refreshed in another task since the
					// first attempt for this request was made in this task. Only refresh it if it
					// wasn’t done yet
					if *access_token_locked == access_token
					{
						log::info!("GitHub App installation access token has possibly expired, \
							requesting a fresh one");

						*access_token_locked =
							AccessToken::new(&self.config, &self.private_key, &self.reqwest_client)
								.await?;
						access_token = access_token_locked.clone();
					}

					// Drop the lock on the access token so other tasks can make requests again
				}

				// Retry the request with the refreshed access token
				request(&self.config, &self.reqwest_client, method, endpoint, body, &access_token)
					.await
			},
			// If the request succeeded or failed with for a different reason than a possibly
			// expired access token, return the result as is
			result => result,
		}
	}

	/// Make an HTTP DELETE request to the GitHub API (for arguments, see [Client::request]).
	#[allow(dead_code)]
	pub async fn delete<S, R>(&self, endpoint: S) -> Result<R, crate::Error>
	where
		S: AsRef<str>,
		R: serde::de::DeserializeOwned,
	{
		self.request(reqwest::Method::DELETE, endpoint, NO_BODY).await
	}

	/// Make an HTTP GET request to the GitHub API (for arguments, see [Client::request]).
	#[allow(dead_code)]
	pub async fn get<S, R>(&self, endpoint: S) -> Result<R, crate::Error>
	where
		S: AsRef<str>,
		R: serde::de::DeserializeOwned,
	{
		self.request(reqwest::Method::GET, endpoint, NO_BODY).await
	}

	/// Make an HTTP HEAD request to the GitHub API (for arguments, see [Client::request]).
	#[allow(dead_code)]
	pub async fn head<S, R>(&self, endpoint: S) -> Result<R, crate::Error>
	where
		S: AsRef<str>,
		R: serde::de::DeserializeOwned,
	{
		self.request(reqwest::Method::HEAD, endpoint, NO_BODY).await
	}

	/// Make an HTTP PATCH request to the GitHub API (for arguments, see [Client::request]).
	#[allow(dead_code)]
	pub async fn patch<S, B, R>(&self, endpoint: S, body: &B) -> Result<R, crate::Error>
	where
		S: AsRef<str>,
		B: serde::Serialize,
		R: serde::de::DeserializeOwned,
	{
		self.request(reqwest::Method::PATCH, endpoint, Some(body)).await
	}

	/// Make an HTTP POST request to the GitHub API (for arguments, see [Client::request]).
	#[allow(dead_code)]
	pub async fn post<S, B, R>(&self, endpoint: S, body: &B) -> Result<R, crate::Error>
	where
		S: AsRef<str>,
		B: serde::Serialize,
		R: serde::de::DeserializeOwned,
	{
		self.request(reqwest::Method::POST, endpoint, Some(body)).await
	}

	/// Make an HTTP PUT request to the GitHub API (for arguments, see [Client::request]).
	#[allow(dead_code)]
	pub async fn put<S, B, R>(&self, endpoint: S, body: &B) -> Result<R, crate::Error>
	where
		S: AsRef<str>,
		B: serde::Serialize,
		R: serde::de::DeserializeOwned,
	{
		self.request(reqwest::Method::PUT, endpoint, Some(body)).await
	}
}

/// When making requests without a request body, we don’t care which type is used to represent it.
/// However, the compiler needs to know some type at compile time. This alias is used in order not
/// to have to spell out the dummy type.
pub const NO_BODY: Option<&()> = None;

/// Internal method for making HTTP requests in the initialization phase.
#[doc(hidden)]
async fn request<S, B, R>(
	config: &Config,
	reqwest_client: &reqwest_middleware::ClientWithMiddleware,
	method: reqwest::Method,
	endpoint: S,
	body: Option<&B>,
	access_token: &AccessToken)
	-> Result<R, crate::Error>
where
	S: AsRef<str>,
	B: serde::Serialize,
	R: serde::de::DeserializeOwned,
{
	// Build the API endpoint URL from the base URL and the endpoint path
	let url = config.base_url.join(endpoint.as_ref()).map_err(crate::Error::ParseUrl)?;
	let mut request = reqwest_client.request(method, url);

	if let Some(body) = body
	{
		// Append the request body if provided
		request = request.json(&body);
	}

	let map_reqwest_error =
		|error| crate::Error::MakeGitHubApiRequest(reqwest_middleware::Error::Reqwest(error));

	let response = request
		// Provide the access token using the Authentication header
		.bearer_auth(access_token)
		// Request the v3 REST API, as recommended by GitHub’s documentation
		.header(reqwest::header::ACCEPT, "application/vnd.github.v3+json")
		// Send the request
		.send().await.map_err(crate::Error::MakeGitHubApiRequest)?;

	// Return an error if there was a client error according to the response’s HTTP status
	if response.status().is_client_error()
	{
		let status_code = response.status();
		let url = response.url().to_owned();

		// Decode the body for debugging purposes
		let response_body = response.text().await.map_err(map_reqwest_error)?;

		return Err(crate::Error::ReceivedGitHubApiClientError{status_code, url, response_body});
	}

	let mut response_body = response
		// Return an error if there was a server error according to the response’s HTTP status
		.error_for_status().map_err(map_reqwest_error)?
		// Read the full response if there was no server error
		.bytes().await.map_err(map_reqwest_error)?;

	// Allow deserializing empty responses as empty dictionaries instead, as empty strings are
	// invalid JSON
	if response_body.is_empty()
	{
		response_body = "{}".as_bytes().into();
	}

	serde_json::from_slice(&response_body).map_err(crate::Error::DecodeGitHubApiResponseBody)
}

/// Verify a webhook event payload by checking the provided signature.
#[doc(hidden)]
fn verify_payload_signature(
	provided_signature: Option<String>,
	payload: &[u8],
	secret: Option<&str>)
	-> Result<(), crate::Error>
{
	let secret = match secret
	{
		Some(secret) => secret,
		// If no secret was configured, accept all payloads
		None =>
		{
			log::warn!("no webhook secret configured, ignoring payload signature (this should be \
				configured for production use)");
			return Ok(());
		}
	};

	// Otherwise, require a valid payload signature. If none is provided, reject the request
	let provided_signature = provided_signature.ok_or(crate::Error::MissingPayloadSignature)?;

	// Only SHA-256 signatures are supported, reject anything else
	let provided_signature = provided_signature.strip_prefix("sha256=")
		.ok_or(crate::Error::InvalidPayloadSignature)?;

	use hmac::Mac as _;

	// Compute the expected signature
	let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes())
		.expect("this call is infallible because HMAC supports keys of arbitrary size");

	mac.update(payload);

	let expected_signature = mac.finalize().into_bytes();
	let expected_signature = hex::encode(&expected_signature);

	// Compare the provided signature with what we expect it to be. Use a secure string wrapper that
	// provides a constant-time equality comparator to prevent timing attacks
	let provided_signature = secstr::SecStr::from(provided_signature);
	let expected_signature = secstr::SecStr::from(expected_signature);

	if provided_signature == expected_signature
	{
		log::debug!("successfully verified payload signature");
		Ok(())
	}
	else
	{
		log::warn!("received payload with invalid signature");
		Err(crate::Error::InvalidPayloadSignature)
	}
}

/// [1]: <https://github.com/seanmonstar/warp/blob/3ff2eaf41eb5ac9321620e5a6434d5b5ec6f313f/examples/todos.rs#L99-L101>
/// [2]: <https://github.com/seanmonstar/warp/blob/3ff2eaf41eb5ac9321620e5a6434d5b5ec6f313f/src/filters/body.rs#L228-L237>
/// [warp] filter allowing us to extract the payload, verify its signature if configured, and decode
/// it from JSON into a struct of the desired type. Returns the decoded payload and a handle to the
/// GitHub API client for further usage as arguments to subsequent handlers in that order. Inspired
/// by the [to-do example][1] and [JSON decode implementation][2] provided by [warp].
///
/// # Arguments
/// - `client`: The handle to the GitHub API client.
pub fn with_validated_payload_and_client<T>(client: Client)
	-> impl warp::Filter<Extract = (T, Client), Error = warp::Rejection> + Clone
where
	T: serde::de::DeserializeOwned + Send,
{
	use warp::Filter as _;

	warp::any()
		// Relay a handle to the client
		.map(move || {client.clone()})
		// Relay the body as raw bytes for payload signature validation and JSON decoding
		.and(warp::body::bytes())
		// Relay the payload signature header if present
		.and(warp::header::optional::<String>("x-hub-signature-256"))
		// Validate the payload signature if configured and decode the body into JSON
		.and_then(
			|client: Client,
				mut bytes: warp::hyper::body::Bytes,
				provided_signature: Option<String>|
			async move
			{
				use warp::Buf as _;

				// Resize the payload buffer view to the size that was actually written
				let bytes = bytes.copy_to_bytes(bytes.remaining());

				// Decode the payload from JSON
				let payload = serde_json::from_slice(&bytes)
					.map_err(crate::Error::DecodePayloadBody)
					.map_err(warp::reject::custom)?;

				// If configured, require a valid payload signature
				verify_payload_signature(provided_signature, &bytes,
					client.config.webhook_secret.as_deref())
						.map_err(warp::reject::custom)?;

				Ok::<_, warp::Rejection>((payload, client))
			})
		// The last call returned the payload and client as a tuple, but we’d like subsequent calls
		// in the filter chain to receive them as top-level arguments and not nested within a single
		// tuple
		.untuple_one()
}

#[doc(hidden)]
#[derive(serde::Serialize)]
struct JwtClaims<'a>
{
	#[serde(rename = "iat")]
	#[serde(with = "chrono::serde::ts_seconds")]
	issued_at: chrono::DateTime<chrono::Utc>,
	#[serde(rename = "exp")]
	#[serde(with = "chrono::serde::ts_seconds")]
	expires_at: chrono::DateTime<chrono::Utc>,
	#[serde(rename = "iss")]
	issuer: &'a str,
}

#[doc(hidden)]
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct AccessToken(String);

impl AccessToken
{
	async fn new(
		config: &Config,
		private_key: &jsonwebtoken::EncodingKey,
		reqwest_client: &reqwest_middleware::ClientWithMiddleware)
		-> Result<Self, crate::Error>
	{
		let now = chrono::Utc::now();

		// Create JWT claims as explained in the documentation [1]
		// [1] https://docs.github.com/en/developers/apps/building-github-apps/authenticating-with-github-apps#authenticating-as-a-github-app
		let jwt_claims = JwtClaims
		{
			// Pretend that the JWT was issued a minute ago to allow for clock drift
			issued_at: now - chrono::Duration::minutes(1),
			// Ask for the JWT to expire in 10 minutes
			expires_at: now + chrono::Duration::minutes(10),
			// Specify that this JWT was issued by our GitHub App
			issuer: &config.app_id.to_string(),
		};

		let jwt_header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);

		// Encode the payload with the GitHub App’s private key to obtain the JWT
		let jwt = jsonwebtoken::encode(&jwt_header, &jwt_claims, &private_key)
			.map_err(crate::Error::CreateJwt)?;
		// We can use the JWT in lieu of a regular access token for the following API requests
		let access_token = AccessToken(jwt);

		// Make a request to the /orgs/{org}/installation API to get the installation ID on the
		// organization
		let get_organization_installation_url =
			format!("orgs/{}/installation", config.organization);
		let response: GitHubAppInstallationResponse = request(config, reqwest_client,
			reqwest::Method::GET, get_organization_installation_url, NO_BODY, &access_token).await
				.map_err(Box::new).map_err(crate::Error::ObtainGitHubAppInstallationToken)?;
		let installation_id = response.id;

		// Make another request to generate an access token we can use for this installation
		let get_installation_access_token_url =
			format!("app/installations/{installation_id}/access_tokens");
		let response: GitHubAppAccessTokenResponse = request(config, reqwest_client,
			reqwest::Method::POST, get_installation_access_token_url, NO_BODY, &access_token).await
				.map_err(Box::new).map_err(crate::Error::ObtainGitHubAppInstallationToken)?;

		log::info!("successfully obtained installation access token for the organization “{}”",
			config.organization);

		Ok(Self(response.token))
	}
}

impl std::fmt::Display for AccessToken
{
	fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result
	{
		write!(formatter, "{}", self.0)
	}
}

/// Response from a request to retrieve the GitHub App installation for a given organization.
#[doc(hidden)]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct GitHubAppInstallationResponse
{
	pub id: u64,
	// We just need the installation ID, so ignore all other fields
}

/// Response from a request to obtain an access token for a given installation of a GitHub App.
#[doc(hidden)]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct GitHubAppAccessTokenResponse
{
	pub token: String,
	// We just need the token itself, so ignore all other fields
}
