# Branch Autoprotector

The Branch Autoprotector watches a GitHub organization and automatically protects the default branch in new repositories.
This service notifies the creator of the default branch of this automatic branch protection setup by filing an issue in the repository.

## Features

- **Protects the default branch of each new repository** in an organization.
  In this way, commits can only be added to the default branch through pull requests with at least one approving review, while direct pushes are disallowed.
- Leverages **GitHub Apps**, which is the recommended way of interfacing with GitHub.
- Immediately reacts to newly created default branches by subscribing to **GitHub webhook events.**
- Automatically **retries failed requests** as to be unaffected by sporadic network issues.
- Automatically **renews the GitHub App installation token** after it expired to be able to run for long periods of time.
- **Verifies the signatures** of incoming webhook payloads to verify that they were actually sent by the GitHub server.
- Written in Rust with **efficiency, robustness, and security** in mind.

## Limitations

- Only the first branch pushed to the repository will be protected but not branches that are created afterward.
  If the default branch is changed later on, it will need to be protected manually.
- Repository administrators and organization owners can still manually change the branch protection settings at any time regardless of how they have been initially set up by this service.
- Branch protection rules are only set up after the first branch is pushed to a new repository.
  This is because empty repositories don’t have a default branch that could be protected via the GitHub API yet.
- This service only supports a single organization at this time.
- Currently, a GitHub Pro subscription is required on GitHub.com to support private repositories.

## Development

ℹ Follow the steps in the next section if you want to [deploy and run this service in production](#installation-and-usage).

The Branch Autoprotector interacts with GitHub as a GitHub App, which you will need to create and configure first.
As this service is written in Rust, you will further need to install the Rust toolchain.
However, you don’t need to follow the deployment steps when you’re just developing and testing locally.

The GitHub server probably won’t be able to deliver webhook events to your local machine, as it’s very likely not to be publicly exposed to the internet for good reasons.
To work around that, we recommend setting up a webhook proxy with [smee.io] for local development, which will forward webhook events to your local system.

1. [**Create and install the GitHub App**](docs/creating-the-github-app.md) with the necessary permissions and event listeners.
   We recommend using a separate GitHub organization and GitHub App for testing.

2. [**Install the Rust toolchain**](docs/installing-the-rust-toolchain.md) on your local system.

3. [**Set up a webhook proxy**](docs/setting-up-a-webhook-proxy-with-smee.io.md) with [smee.io].

4. In the settings of your GitHub App under *General,* **update the *Webhook URL* field** (which still contains a placeholder URL) with the webhook proxy URL that you generated on [smee.io].

5. [**Configure this service**](docs/configuring-this-service-for-development.md) for development.

6. **Generate the source code documentation** if you’d like to start working on this service:

   ```shell
   $ cargo doc
   ```

   Then, open `target/docs/branch_autoprotector/index.html` in your browser to have a look at the documentation.

7. **Build and run the service** from the root of your working copy of this repository:

   ```shell
   $ RUST_LOG=debug cargo run
   ```

Your local machine will now receive and handle repository creation events in the organization the GitHub App has been installed to 🚀.
To test this, create a new repository in your organization and push some content to it—you should see a new issue being created!
(Note that the issue won’t be created as long as the repository is empty, as it doesn’t have a default branch that could be protected yet.)

## Installation and usage

The Branch Autoprotector interacts with GitHub as a GitHub App, which you will need to create and configure first.
The service is designed to be installed on Debian systems and run as a `systemd` service.
We don’t provide official builds at this time, but you can build a Debian package on your own with a few steps using the Rust toolchain.

1. [**Create and install the GitHub App**](docs/creating-the-github-app.md) with the necessary permissions and event listeners.

2. **Ensure that the following Debian packages are installed** on the system on which you’d like to build the Debian package (that could be your target system or another system with the same Debian version).
   These packages will be needed to build this service:

   ```shell
   $ sudo apt install curl build-essential pkg-config libssl-dev
   ```

3. [**Install the Rust toolchain**](docs/installing-the-rust-toolchain.md) including `cargo deb` on the same system.

4. Clone this repository and **build the Debian package** with `cargo deb`:

   ```shell
   $ git clone https://github.com/branch-autoprotector/branch-autoprotector
   # You can skip this step if you prefer building from the main branch
   $ git switch --detach <latest release tag>
   $ cd branch-autoprotector
   $ cargo deb
   ```

   The resulting Debian package can then be found in `target/debian/branch-autoprotector_<version>_<architecture>.deb`.

5. **Deploy this service** by copying the Debian package to the target system and installing it:

   ```shell
   $ sudo apt install ./branch-autoprotector_<version>_<architecture>.deb
   ```

6. [**Configure this service**](docs/configuring-this-service-for-production-use.md) for production use.

7. [**Set up a reverse proxy with Nginx**](docs/setting-up-a-reverse-proxy-with-nginx.md) serving the Branch Autoprotector at a publicly accessible location.

8. In the settings of your GitHub App under *General,* **update the *Webhook URL*** field (which still contains a placeholder URL) with the URL that you configured in Nginx.

9. **Run and enable this service.**
   This service runs as the `github` user you created earlier [when you configured this service](docs/configuring-this-service-for-production-use.md) and not as `root`:

   ```shell
   $ sudo systemctl enable --now branch-autoprotector
   ```

   `systemd` ensures that services persist across sessions, that is, the service won’t terminate when you close your connection.
   By enabling it, this service will also be started automatically when your system boots.
   The `--now` flag lets the service start immediately and not just the next time you reboot.

Your server will now receive and handle repository creation events in the organization the GitHub App has been installed to 🚀.
To test this, create a new repository in your organization and push some content to it—you should see a new issue being created!
(Note that the issue won’t be created as long as the repository is empty, as it doesn’t have a default branch that could be protected yet.)

### Notes for new developers

If you’d like to adjust or extend this service, start by looking at [`main.rs`](src/main.rs).
There, the route for handling incoming webhook events is defined as well as the handler for branch creation events.

All functionality related to making calls to the GitHub API, GitHub Apps authentication, and verifying payloads from webhook events delivered by GitHub is encapsulated in the `github_api` module.
If you want to make calls to API endpoints not yet implemented, it’s likely that you won’t need to touch that module though.
Instead, you can add new request and response types in [`models.rs`](src/models.rs), and the implementation of the GitHub API client will be able to handle those requests without relinquishing static type checking.

## Notes concerning the assignment

- No particular programming language, technology stack, or similar was requested.
  For this reason, I assumed the client to be interested in a service that’s reliable, secure, and has a small resource footprint on the server.
  To achieve that, I decided to build this service in Rust.
  I also presumed that the client requested robustness against sporadic network issues, automatic renewal of GitHub App installation tokens (to ensure the service can run for a long time unattended), and built-in webhook payload signature verification.
  Though there is no official GitHub API binding like OctoKit for Rust yet, I presumed that the client was aware of the trade-offs and finally decided to ask for a Rust implementation.
- For deployment, I decided to install a `systemd` service (using a Debian package), which I find to be a lightweight way to manage services and make them persist across sessions and reboots nicely.
- Using a personal access token and webhooks configured on the organization level rather than GitHub Apps would certainly have yielded a much simpler implementation.
  My assumption was, however, that this integration is meant to be used in production, and I don’t think that running integrations using personal access tokens from user accounts is a good choice.
  For this reason, I decided to implement this service using GitHub Apps.
- The client explicitly asked for this this service to set up a branch protection rule whenever new repositories are created.
  I decided to slightly deviate from this by configuring the branch protection rule only when the default branch is pushed for the first time.
  The reason is that prior to pushing the first branch, the repository is empty, which would mean that we’d have to guess the name of the default branch that’s going to be used.
  Also, the GitHub API refuses setting up branch protection rules if there are no branches yet.
  I believe this adjustment to be in the client’s interests.

## Resources used while developing this service

- the [docs.rs](https://docs.rs) pages of all Rust crates used for this project as well as `cargo deb`
- the official GitHub documentation on [GitHub Apps authentication](https://docs.github.com/en/developers/apps/building-github-apps/authenticating-with-github-apps) and [securing webhooks](https://docs.github.com/en/developers/webhooks-and-events/webhooks/securing-your-webhooks)
- the [GitHub REST API reference](https://docs.github.com/en/rest)
- the [GitHub REST API OpenAPI description](https://github.com/github/rest-api-description)
- the documentation of [Octokit](https://github.com/octokit/octokit.js) and the [GitHub App toolset for Octokit](https://github.com/octokit/app.js)
- [GitHub discussion on limitations of the Ruby version of Octokit](https://github.com/octokit/octokit.rb/discussions/1096)

[smee.io]: https://smee.io/
