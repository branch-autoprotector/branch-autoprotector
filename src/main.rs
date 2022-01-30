#[doc(hidden)]
mod config;
#[doc(hidden)]
mod error;
pub mod github_api;
#[doc(hidden)]
mod models;

pub use config::Config;
pub use error::Error;
pub use models::*;

#[tokio::main]
async fn main() -> anyhow::Result<()>
{
	pretty_env_logger::init();

	// Read the config file
	let config = Config::from_file("config.yaml")?;

	// Initialize a new GitHub API client using the GitHub App created for this service
	let github_api_client = github_api::Client::from_config(config.github_api).await?;

	use warp::Filter as _;

	let ref_creation_event_route =
		// Only listen for requests to the root path
		warp::path::end()
		// Only listen for POST requests
		.and(warp::post())
		// Only listen for ref creation events
		.and(warp::header::exact_ignore_case("x-github-event", "create"))
		// Reject payloads larger than 256 kB, which should be enough for all valid requests
		.and(warp::body::content_length_limit(256 * 1024))
		// Retrieve and validate the payload and pass it on along with the GitHub API client
		.and(github_api::with_validated_payload_and_client(github_api_client))
		// Forward request to request handler
		.and_then(handle_ref_creation_event);

	let routes = ref_creation_event_route
		.recover(handle_rejection);

	log::info!("listening for incoming webhook events on 127.0.0.1:2342");
	warp::serve(routes).run(([127, 0, 0, 1], 2342)).await;

	Ok(())
}

/// Request handler for valid ref creation events.
///
/// # Arguments
/// - `payload`: The decoded webhook event payload.
/// - `github_api_client`: A handle to the GitHub API client.
async fn handle_ref_creation_event(
	payload: RefCreationEventPayload,
	github_api_client: github_api::Client)
	-> Result<impl warp::Reply, std::convert::Infallible>
{
	let branch_name = payload.ref_;
	let default_branch_name = payload.master_branch;

	// Ignore all actions other than the creation of a branch. Also, if the newly created branch is
	// not the default branch, this isn’t the first branch being created, so don’t set up branch
	// protection rules either. In both cases, return a successful HTTP response
	if payload.ref_type != RefType::Branch || branch_name != default_branch_name
	{
		log::debug!("unrelated ref creation event, ignoring");

		let message = "not listening to this ref creation event";
		let response = warp::reply::json(&InfoResponse{info: message});

		return Ok(warp::reply::with_status(response, warp::http::StatusCode::OK));
	}

	let creator_name = payload.sender.login;
	let organization_name = payload.repository.owner.login;
	let repository_name = payload.repository.name;

	log::info!("repository “{repository_name}” was created in organization “{organization_name}” \
		with a new default branch “{branch_name}”");

	// Protect the default branch and inform about this in an issue in a separate task so as to
	// immediately acknowledge the webhook event without blocking
	tokio::spawn(
		async move
		{
			// Protect the new default branch by disallowing users from pushing directly (including
			// administrators) and requiring at least one pull request review
			let protect_branch_request = ProtectBranchRequest
			{
				required_status_checks: None,
				enforce_admins: Some(true),
				required_pull_request_reviews: Some(RequiredPullRequestReviews{}),
				restrictions: None,
			};

			if let Err(error) = github_api_client.put::<_, _, IgnoreResponse>(
				format!("repos/{organization_name}/{repository_name}/branches/{branch_name}\
					/protection"),
				&protect_branch_request).await
			{
				log::error!("could not set up branch protection rule for branch “{branch_name}” in \
					repository “{repository_name}”");
				log::error!("{:?}", anyhow::Error::from(error));
				return;
			}

			log::info!("set up branch protection rule for branch “{branch_name}” in repository \
				“{repository_name}”");

			// Notify the user triggering the branch creation event of the newly set-up branch
			// protection rules
			let issue_title = "Branch protection automatically set up";
			let issue_body = format!(
				"@{creator_name}: The default branch [`{branch_name}`](../tree/{branch_name}) was \
				automatically protected to comply with our corporate policies. Please submit pull \
				requests in order to contribute changes, as direct pushes to this branch are not \
				allowed. Every pull request needs to be approved by at least one person before it \
				can be merged. Please review the [branch protection rules in the repository \
				settings](../settings/branches) and extend them as necessary.\
				\n\
				\n\
				This issue is just for your information and can be closed after reviewing the \
				branch protection rules.");

			let create_issue_request_body = CreateIssueRequest
			{
				title: issue_title,
				body: Some(&issue_body),
			};

			let created_issue: CreateIssueResponse = match github_api_client.post(
				format!("repos/{organization_name}/{repository_name}/issues"),
				&create_issue_request_body).await
			{
				Ok(created_issue) => created_issue,
				Err(error) =>
				{
					log::error!("could not notify repository creator about new branch protection \
						rules set up for repository “{repository_name}”");
					log::error!("{:?}", anyhow::Error::from(error));
					return;
				}
			};

			log::info!("created issue informing about branch protection: {}",
				created_issue.html_url);
		});

	// Acknowledge the successful receipt of this webhook event as quickly as possible
	let message = "creating branch protection rules and notifying creator of the default branch";
	let response = warp::reply::json(&InfoResponse{info: message});

	Ok(warp::reply::with_status(response, warp::http::StatusCode::OK))
}

/// Request handler for all requests that were rejected previously.
///
/// # Arguments
/// - `error`: Reasons for why this request was rejected by all routes.
async fn handle_rejection(error: warp::Rejection)
	-> Result<impl warp::Reply, std::convert::Infallible>
{
	let status_code;
	let message;

	if error.is_not_found()
	{
		status_code = warp::http::StatusCode::NOT_FOUND;
		message = "not found";
	}
	else if let Some(_) = error.find::<warp::reject::MethodNotAllowed>()
	{
		status_code = warp::http::StatusCode::METHOD_NOT_ALLOWED;
		message = "method not allowed";
	}
	else if let Some(_) = error.find::<warp::reject::PayloadTooLarge>()
	{
		status_code = warp::http::StatusCode::BAD_REQUEST;
		message = "payload too large";
	}
	else if let Some(_) = error.find::<warp::reject::MissingHeader>()
	{
		status_code = warp::http::StatusCode::BAD_REQUEST;
		message = "missing webhook event header";
	}
	// Don’t treat events that we don’t react to as errors and report a 200 OK instead
	else if let Some(_) = error.find::<warp::reject::InvalidHeader>()
	{
		status_code = warp::http::StatusCode::OK;
		message = "not listening to this webhook event";
	}
	else if let Some(crate::Error::DecodePayloadBody(_)) = error.find()
	{
		status_code = warp::http::StatusCode::BAD_REQUEST;
		message = "malformed payload body";
	}
	else if let Some(crate::Error::MissingPayloadSignature) = error.find()
	{
		status_code = warp::http::StatusCode::BAD_REQUEST;
		message = "missing payload signature";
	}
	else if let Some(crate::Error::InvalidPayloadSignature) = error.find()
	{
		status_code = warp::http::StatusCode::BAD_REQUEST;
		message = "invalid payload signature";
	}
	// If users are able to trigger errors we did not anticipate, log the error chain so we can
	// inspect this more closely later
	else
	{
		status_code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
		message = "internal server error";

		log::error!("unhandled error: {:#?}", error);
	}

	let response = match status_code.is_success()
	{
		true => warp::reply::json(&InfoResponse{info: message}),
		false => warp::reply::json(&ErrorResponse{error: message}),
	};

	Ok(warp::reply::with_status(response, status_code))
}

/// Response type acknowledging successfully handled webhook events (serialized to JSON).
#[derive(serde::Serialize)]
struct InfoResponse<'a>
{
	/// Info message with human-readable information about how this request was handled.
	info: &'a str,
}

/// Response type informing about errors while handling webhook events (serialized to JSON).
#[derive(serde::Serialize)]
struct ErrorResponse<'a>
{
	/// Error message with a human-readable explanation as to why this request failed.
	error: &'a str,
}
