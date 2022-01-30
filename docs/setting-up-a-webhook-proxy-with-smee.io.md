# Setting up a webhook proxy with [smee.io]

When testing locally, the GitHub.com API doesn’t have a direct way to reach your local machine from the internet.
This can be solved by using [smee.io] as a proxy to forward GitHub.com webhook event payloads to your local service.

1. [Start a new smee.io channel](https://smee.io/new).
   This will bring you to a page with instructions:

   ![Create a new smee.io channel to get a webhook proxy URL for local development](screenshots/smee.io.png)
   
2. Note the webhook proxy URL shown at the top of the page (https://smee.io/dU1gjQNNvoJW6 in this example, it will be different for you).
   
3. Install the `smee` client on your machine if you don’t have it yet:

   ```shell
   $ npm install --global smee-client
   ```

4. Start the `smee` client, asking it to listen to the webhook proxy URL that was generated for you and to forward webhook events to http://127.0.0.1:2342/, which is where the Branch Autoprotector will be listening.
   Don’t forget to replace `<webhook_proxy_url>` with the webhook proxy URL you were assigned:
   
   ```shell
   $ smee --url <webhook_proxy_url> --port 2342
   ```
   
   Keep the `smee` client running for as long as you’re testing the Branch Autoprotector.
   It’s advisable to run this command in a `tmux` or `screen` session to prevent it from terminating when you accidentally close your terminal.

[smee.io]: https://smee.io/