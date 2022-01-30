# Configuring this service for development

As mentioned earlier, we recommend that you create a separate organization and GitHub App for testing.

1. **Move the private key** of your GitHub App ([which you obtained earlier](creating-the-github-app.md)) to a safe location on your hard drive.
   
3. **Copy the configuration file.**
   For this, change into your working copy of this repository and create a copy of `config.example.yaml` named `config.yaml` in the same directory.

4. **Restrict access to both files,** as they contain secrets, especially if you share your system with other users.
   Make sure that the files are not readable by any other user:

   ```shell
   $ chmod 600 /path/to/config.yaml /path/to/key.pem
   ```
   
5. **Edit `config.yaml`.**
   Set the organization name, GitHub App ID, and webhook secret to the values you obtained when creating your GitHub App.
   Also, make sure that the file path of the private key of your GitHub App is correct (we recommend using an absolute path).