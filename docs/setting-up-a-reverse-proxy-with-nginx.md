# Setting up a reverse proxy with Nginx

In order to expose this service to the public internet, we recommend using Nginx and a reverse proxy configuration.
To secure this setup, we recommend listening on HTTPS rather than HTTP.
For this, you will need to obtain a TLS certificate for your server name.

1. **Obtain a TLS certificate** for your server name.
   In most companies, you will need to request it from a company-internal service for issuing and renewing TLS certificates.
   In the open-source community, [Let’s Encrypt](https://letsencrypt.org/) is a popular choice.

2. Get started with this **basic Nginx reverse proxy configuration:**

   ```nginx
   server
   {
       listen 80;
       listen [::]:80;
       server_name <server_name>;
   
       return 301 https://$server_name$request_uri;
   }
   
   server
   {
       listen 443 ssl http2;
       listen [::]:443 ssl http2;
       server_name <server_name>;
       ssl_certificate /path/to/fullchain.pem;
       ssl_certificate_key /path/to/privkey.pem;
       ssl_trusted_certificate /path/to/chain.pem;
       
       location /
       {
           proxy_pass http://localhost:2342;
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
       }
   }
   ```

   Replace `<server_name>` with the actual name of your server (for example, `branch-autoprotector.example.com`) and the paths beginning with `/path/to` with the actual locations of your certificate’s private key, chain, and full chain on your system.

3. **Review the [recommended SSL configuration for Nginx](https://ssl-config.mozilla.org/)** provided by Mozilla based on your needs for backward compatibility.
   Copy the SSL-related settings into the configuration block listening on port 443, leaving the `location` directive untouched.
   In doubt, you may want to read the [official Nginx guide for configuring HTTPS servers](https://nginx.org/en/docs/http/configuring_https_servers.html) in addition to that.

4. **Add the two `server` blocks to the `http` section** of your `/etc/nginx.conf`.
   On some distributions, this file is just a skeleton, and you are expected to put these `server` blocks into a new file (for example, `branch-autoprotector`) in `/etc/nginx/sites-available`, which you then symbolically link into `/etc/nginx-sites-enabled`:

   ```shell
   $ sudo ln -s /etc/nginx/sites-available/branch-autoprotector /etcx/nginx/sites-enabled/
   ```

5. **Reload Nginx** for the changes to take effect:

   ```shell
   $ sudo systemctl reload nginx
   ```

6. If you are unsure of your SSL and TLS configuration, request the [Mozilla TLS Observatory](https://observatory.mozilla.org/) to **scan your server for security issues** and recommendations.