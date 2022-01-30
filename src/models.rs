/// Partial user data model as returned in responses from the GitHub API.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct User
{
	/// The user’s handle.
	pub login: String,
	// We don’t need the other fields, so ignore them
}

/// Partial repository data model as returned in responses from the GitHub API.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Repository
{
	/// The name of the repository.
	pub name: String,
	/// Handle of the user or organization owning the repository.
	pub owner: User,
	// We don’t need the other fields, so ignore them
}

/// Type of a Git ref object.
#[derive(Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefType
{
	Branch,
	Tag,
}

/// Webhook event payload for ref creation events as provided by the GitHub server.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RefCreationEventPayload
{
	/// The `git ref` resource.
	#[serde(rename = "ref")]
	pub ref_: String,
	/// The type of Git ref object created in the repository.
	pub ref_type: RefType,
	/// The name of the repository’s default branch (usually `main`).
	pub master_branch: String,
	/// The repository for which this event is reported.
	pub repository: Repository,
	/// Record of the user causing this event.
	pub sender: User,
	// We don’t need the other fields, so ignore them
}

/// Partial data model for the parameters needed to make a GitHub API request to protect a branch.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProtectBranchRequest
{
	/// Field currently unsupported, so leave this at `None`.
	pub required_status_checks: Option<UnsupportedField>,
	/// Enforce all configured restrictions for administrators. Set to `Some(true)` to enforce
	/// required status checks for repository administrators. Set to `None` to disable.
	pub enforce_admins: Option<bool>,
	/// Require at least one approving review on a pull request, before merging. Set to `None` to
	/// disable.
	pub required_pull_request_reviews: Option<RequiredPullRequestReviews>,
	/// Field currently unsupported, so leave this at `None`.
	pub restrictions: Option<UnsupportedField>,
	// We don’t need to set the other optional fields, so ignore them
}

/// Partial data model for the parameters needed to make a GitHub API request to protect a branch.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RequiredPullRequestReviews
{
	// We currently don’t need any of the optional fields, so ignore them
}

/// Partial data model for the parameters needed to make a GitHub API request to create a new issue.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateIssueRequest<'a>
{
	/// The title of the issue.
	pub title: &'a str,
	/// The contents of the issue.
	pub body: Option<&'a str>,
	// We don’t need to set the optional fields, so ignore them
}

/// Partial data model for the response of the GitHub API to a request to create a new issue.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateIssueResponse
{
	/// User-facing URL of the created issue.
	pub html_url: url::Url,
	// We don’t need the other fields, so ignore them
}

/// A field that is currently unsupported and needs to be set to `None` currently.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UnsupportedField;

/// Data model representing a response we’re going to ignore.
#[derive(Debug, serde::Deserialize)]
pub struct IgnoreResponse
{
}
