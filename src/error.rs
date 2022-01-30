/// All errors that may occur during initialization or while handling requests.
#[derive(Debug, thiserror::Error)]
pub enum Error
{
	#[error("could not read config file")]
	ReadConfigFile(#[source] std::io::Error),
	#[error("could not parse config file")]
	ParseConfigFile(#[source] serde_yaml::Error),

	#[error("could not create HTTP client")]
	CreateHttpClient(#[source] reqwest::Error),

	#[error("could not read private GitHub App key file")]
	ReadPrivateGitHubAppKeyFile(#[source] std::io::Error),
	#[error("could not parse private GitHub App key file")]
	ParsePrivateGitHubAppKeyFile(#[source] jsonwebtoken::errors::Error),
	#[error("could not create JWT")]
	CreateJwt(#[source] jsonwebtoken::errors::Error),
	#[error("could not obtain GitHub App installation access token")]
	ObtainGitHubAppInstallationToken(#[source] Box<crate::Error>),

	#[error("could not parse URL")]
	ParseUrl(#[source] url::ParseError),
	#[error("could not make GitHub API request")]
	MakeGitHubApiRequest(#[source] reqwest_middleware::Error),
	#[error("received GitHub API client error (status code {status_code}): {response_body}")]
	ReceivedGitHubApiClientError
	{
		status_code: reqwest::StatusCode,
		url: url::Url,
		response_body: String,
	},
	#[error("could not decode GitHub API response body")]
	DecodeGitHubApiResponseBody(#[source] serde_json::Error),

	#[error("could not decode payload body")]
	DecodePayloadBody(#[source] serde_json::Error),
	#[error("missing payload signature")]
	MissingPayloadSignature,
	#[error("invalid payload signature")]
	InvalidPayloadSignature,
}

// Allow this crate’s error type to be used for failed HTTP responses
impl warp::reject::Reject for Error
{
}
