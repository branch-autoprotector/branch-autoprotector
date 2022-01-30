# Creating the GitHub App

1. Go to your organization’s *Settings* page.
   Under *Developer settings* → *GitHub Apps*, click the *New GitHub App* button:

   ![*New GitHub App* button in the organization settings](screenshots/github-apps-1.png)

2. Give your new GitHub App a descriptive name and a URL where users with questions about this service might get valuable information:

   ![Set the name and homepage URL of your new GitHub App](screenshots/github-apps-2.png)

3. For now, enter a placeholder webhook URL under *Webhook.*
   We will set it to a proper value later.
   Generate a secure webhook secret as explained in [GitHub’s documentation on that topic](https://docs.github.com/en/developers/webhooks-and-events/webhooks/securing-your-webhooks#setting-your-secret-token).
   We’ll need it later to configure this service.

   ![Set a placeholder webhook URL and generate a secure webhook secret](screenshots/github-apps-3.png)

4. Under *Repository permissions,* we need

   - *Administration* to set to *Read & write* in order to be able to configure branch protection rules,
   - *Contents* set to *Read-only* in order to be notified of newly created branches, and
   - *Issues* set to *Read & write* in order to be able to create a new issue:

   ![Set the repository permissions required by this service](screenshots/github-apps-4.png)

5. Under *Subscribe to events,* select *Create* in order to be notified of new branches:

   ![Subscribe to the branch or tag creation event](screenshots/github-apps-5.png)

6. Create the GitHub App:

   ![Create the GitHub App](screenshots/github-apps-6.png)

7. You will be redirected to the settings page of your new GitHub App.
   Note the GitHub App ID, which is shown at the top of the page and which we’ll need to configure this service later:

   ![The GitHub App ID is shown at the top of the settings page of your new GitHub App](screenshots/github-apps-7.png)

7. From the settings page, click *Generate a private key:*
  
   ![Generate a private key for your new GitHub App](screenshots/github-apps-8.png)
   
8. Store the key in a safe location on your disk, as it will be needed by this service later.

9. Install the new GitHub App to your organization by going to the *Install app* tab to the left of your GitHub App’s settings page and clicking *Install* next to the name of your organization:

   ![Install your new GitHub App to your organization](screenshots/github-apps-9.png)