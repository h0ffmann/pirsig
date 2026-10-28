# The "Pirsig bootstrap" GitHub App

`psg bootstrap gh` signs you in to this app and indexes your GitHub FOSS work into your
brain (design: [docs/design/github-bootstrap.md](../docs/design/github-bootstrap.md)).
The app is registered **once**, by the maintainer; everyone else just signs in.

[`manifest.json`](manifest.json) is generated from `crates/pirsig-github/src/manifest.rs`
(a test fails if they drift):

```sh
cargo run -q -p pirsig-github --bin pirsig-gh-app > github-app/manifest.json
```

Permissions are read-only: `metadata`, `contents` (public READMEs, opt-in), and
`pull_requests` (counting your PRs in org repos). No webhooks, no events, no server.

## Register the app (maintainer, once)

This needs a browser signed in to the GitHub account (or org) that will own the app.

1. Write the registration page and open it:

   ```sh
   cargo run -q -p pirsig-github --bin pirsig-gh-app register > /tmp/register.html            # your account
   cargo run -q -p pirsig-github --bin pirsig-gh-app register ouroboric-dev > /tmp/register.html  # or an org
   xdg-open /tmp/register.html   # macOS: open
   ```

   App names are global on GitHub. If "Pirsig bootstrap" is taken, set another name:
   `PIRSIG_APP_NAME="psg bootstrap (h0ffmann)" cargo run ... register`.

2. Click **Create the GitHub App**, review the permissions GitHub shows, confirm.

3. GitHub redirects to the repo URL with `?code=XXXX` in the address bar. Within one
   hour, exchange it (no auth needed, the code is single-use):

   ```sh
   curl -sX POST https://api.github.com/app-manifests/XXXX/conversions > pirsig-app.json
   ```

   The response has `id`, `slug`, `client_id`, `client_secret`, `pem` and
   `webhook_secret`. **Never commit this file.** Keep `client_secret` and `pem` in a
   password manager; the CLI does not need them. Then `shred -u pirsig-app.json`.

4. In the app's settings (`github.com/settings/apps/<slug>`), tick **Enable Device
   Flow** (the manifest cannot set it) and, under *Optional features*, keep
   **user-to-server token expiration** on.

5. Put the public `client_id` in Pirsig (`PIRSIG_GITHUB_CLIENT_ID`, default compiled into the
   binary in `crates/pirsig-github`). A client id is not a secret: the device flow uses it
   without the client secret.

## Use it (everyone)

```sh
psg bootstrap gh                 # opens github.com/login/device, you type the code shown
```

1. **Install** the app on your account, choosing *All repositories* or the ones you
   want: `https://github.com/apps/<slug>/installations/new`.
2. For **private org repos** you contributed to, an owner of that org has to install the
   app on it (GitHub sends them a request when you try). Without that, those repos are
   simply not visible to the app and are skipped; public org repos work regardless.
3. Sign in with the device flow. The token is a user-to-server token: it can only see
   what both you and the installation can see, expires in 8 hours, and is refreshed
   with a refresh token stored in your OS keychain (`~/.config/pirsig/` as a fallback,
   mode 600).

Revoke any time: *Settings → Applications → Authorized GitHub Apps → psg bootstrap*.
