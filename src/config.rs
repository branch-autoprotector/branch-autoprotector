#[derive(serde::Deserialize)]
/// Top-level configuration of this application.
///
/// Currently, only GitHub-API-specific configuration options are available, but this can be
/// extended as to include unrelated configuration options when needed.
pub struct Config
{
	/// Configuration options specific to the GitHub API and authentication.
	pub github_api: crate::github_api::Config,
}

impl Config
{
	/// Attempt to read and parse the configuration from a YAML file.
	///
	/// # Arguments
	/// `path`: Path to the configuration file in YAML format.
	pub fn from_file<P>(path: P) -> Result<Self, crate::Error>
	where
		P: AsRef<std::path::Path>
	{
		let file = std::fs::File::open(&path).map_err(crate::Error::ReadConfigFile)?;
		serde_yaml::from_reader(&file).map_err(crate::Error::ParseConfigFile)
	}
}
