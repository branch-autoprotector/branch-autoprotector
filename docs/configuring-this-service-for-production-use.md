# Configuring this service for production use

Perform these steps on the target server, which you installed this service to earlier.

1. **Create a dedicated Unix user called `github`** that this system will run as:
  
   ````shell
   $ sudo useradd -m github
   ````
   
   We recommend not to run this service as `root`.
   
1. **Copy the private key** of your GitHub App ([which you obtained earlier](creating-the-github-app.md)) to the target machine.
   We recommend storing it in `/etc/branch-autoprotector/key.pem`.

3. **Copy the configuration file.**
   For this, change into `/etc/branch-autoprotector` and create a copy of `config.example.yaml` named `config.yaml` in the same directory.

4. **Restrict access to both files,** as they contain secrets.
   Make sure that the files are owned by the `github` user and not readable by any other user:

   ```shell
   $ sudo chown github:github /etc/branch-autoprotector/{config.yaml,key.pem}
   $ sudo chmod 600 /etc/branch-autoprotector/{config.yaml,key.pem}
   ```

5. **Edit `config.yaml`.**
   Set the organization name, GitHub App ID, and webhook secret to the values you obtained when creating your GitHub App.
   Also, make sure that the file path of the private key of your GitHub App is correct (we recommend using an absolute path).