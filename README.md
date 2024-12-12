# Nanowatchrs

This is a super small and simple status page. It is meant to be lean when served, and lean when ran. It uses Rust to poll the status of the services, and then creates static HTML files that can be served as static assets (who woulda thunk).

Very minimal CSS and JS is used to make the page look and behave as necessary, but JS is not required for the page to function.

This is meant to be launched as a cron job, or as a repeated task, or something similar, to keep the status page up to date. Rust is used to allow for running as a native binary, or inside a wasm runtime.

Demo site: [status.isaacdonaldson.com](https://status.isaacdonaldson.com)

# Configuration

All configuration and history files are stored in the `config/` directory, so make sure to have that directory created already. This repository contains an example `config/` directory that you can use as a starting/reference point.

The status page is configured using a `config.json` file. The config file has 3 parts/keys: `"settings"`, `"checks"`, and `"incidents"`.

Below is an explanation of each of the configuration options.

## Settings

```json
{
  "settings": {
    "site": {
      "name": "Your Site Name",
      "description": "Your site description",
      "url": "https://your-domain.com",
      "logo": "https://your-domain.com/logo.svg"
    },
    "page": {
      "title": "Status Page Title",
      "header": "Status Page Header",
      "header_link": "https://your-domain.com",
      "subheader": "Status Page Subheader Details"
    }
  }
}
```

The `"site"` settings represent general settings. These are used in the meta tags in the header of the page.

- `name`: Your site or service name
- `description`: Brief description of your service
- `url`: Main website URL
- `logo`: URL to your logo image, displayed as the favicon

The `"page"` settings represent the settings for the page itself. These are used in the title and header of the page.

- `title`: Browser tab title
- `header`: Main heading displayed on the status page
- `header_link`: URL to link the header to, if absent will not link to anything
- `subheader`: Subheading displayed under the main heading

## Service Checks

The `checks` array defines the services to monitor. Three types of checks are supported, but they share the same structure:

- `name`: Display name for the service. Important as it's used as the identifier
- `description`: (Optional) Service description. Displayed under the name on the status section
- `type`: Check type (`http`, `ping`, or `port`)
- `target`: URL, hostname, or IP to check (context dependent)
- `page_link`: (Optional) URL to service documentation or information
- `expected_status`: (HTTP check only) Expected response code
- `port`: (Port check only) Port number to test
- `timeout_ms`: Maximum time to wait for response in milliseconds. Danger (Potential Outage or Issue) will be reported if timeout is reached

### HTTP Check

```json
{
  "name": "API Service",
  "description": "Optional service description",
  "type": "http",
  "target": "https://api.example.com",
  "page_link": "https://docs.example.com",
  "expected_status": 200,
  "timeout_ms": 5000
}
```

### Ping Check

```json
{
  "name": "Domain Check",
  "type": "ping",
  "target": "example.com",
  "page_link": "https://example.com",
  "timeout_ms": 5000
}
```

_Note_: Ping checks might not be supported on some setups (like Cloudflare Workers or AWS Lambdas).

### Port Check

```json
{
  "name": "Database",
  "type": "port",
  "target": "db.example.com",
  "port": 5432,
  "timeout_ms": 5000
}
```

_Note_: Port checks might not be supported on some setups (like cloudflare workers).

## Incidents

The `incidents` array allows you to document service incidents. These will show up in the incidents section at the bottom of the page. Incidents are not automated and must be added, removed, updated, and resolved manually.

```json
{
  "title": "Incident Title",
  "description": "Detailed incident description",
  "status": "Ongoing|Resolved",
  "display_date": "2024-10-23",
  "started_at": "2024-10-23 10:00:00",
  "resolved_at": "2024-10-23 11:00:00"
}
```

All the fields are required for an incident to be displayed.

- `title`: Incident title
- `description`: Detailed incident information. Linebreaks and tabs are supported
- `status`: Current status, `Ongoing|ongoing` (yellow dot) or `Resolved|resolved` (green dot). Any other string will not display a dot
- `display_date`: Date to display (YYYY-MM-DD)
- `started_at`: Incident start time (YYYY-MM-DD HH:MM:SS). Not displayed and for reference only
- `resolved_at`: Incident resolution time (YYYY-MM-DD HH:MM:SS). Not displayed and for reference only

# History

Most of the history part is automated, but you can manually change the data if you'd like. The `"uptime"` field is optional, but if you specify it then it will overwrite the calculated uptime. You can also hand edit any of the history files, they can be found in the `config/` directory where each check has its own file.

Some examples:

- `Backend API` -> `config/Backend_API_history.json`
- `Domain` -> `config/Domain_history.json`
- `Database Conntection` -> `config/Database_Connection_history.json`

# Running

NanoWatchrs is designed to run as a scheduled task rather than a continuous service. This approach provides flexibility and works well with:

- System-level schedulers (like cron)
- Cloud provider schedulers (AWS Lambda, Google Cloud Functions, etc.)
- CI/CD pipeline schedulers

Instead of defining check frequencies in the configuration file, you control the frequency through your scheduler. Each check can be run independently using the check's name.

For example, to run a specific check:

```bash
nanowatchrs --check "Backend API"
# Or short form
nanowatchrs -c "Backend API"
```

To run all checks:

```bash
nanowatchrs --all
# Or short form
nanowatchrs -a
```

This approach gives you complete control over:

- How frequently each check runs
- Which scheduler to use
- Which Resources to use
- Cost optimization (especially in cloud environments)

## GitHub Actions

I previously ran the demo with GitHub Actions and had it run every 30 mins, but since it is not actually checking anything, I thought that it was a bit wasteful and changed it to once a day. Since the ping check cannot be run in GitHub actions, I have removed it, but the configuration examples are still valid. Below is a guide on how you can set it up. (The actions are still here as an example, just disabled in GitHub).

The neat thing about using GitHub Actions to run this is that you can schedule it with a cron, and push to the repository to trigger a deploy action if you'd like.

Here's an example workflow that builds the project, runs checks, and commits any changes:

_Note_: There might be a way to use the new GitHub Personal Access Tokens, but I am not sure how to do that yet.

Create a Personal Access Token: Settings > Developer Settings > Personal Access Tokens (Classic) > Generate new token. Give it the `repo` scope.

Then go to your repository > Settings > Secrets and Variables > New Secret. And name it accordingly with the secret pasted.

You can check out the `.github/workflows` directory for example workflows, but here's a simple one where your PAT secret is named `AUTOMATED_STATUS_CHECKS`:

```yaml
name: Backend Check

on:
  # Allow manually triggering the workflow from the Actions tab in GitHub
  workflow_dispatch:
  schedule:
    # Every hour, on the hour
    - cron: "0 * * * *"

env:
  RUST_TOOLCHAIN: stable
  CHECK_NAME: Backend API

jobs:
  backend_check:
    name: Backend Status Check
    runs-on: ubuntu-latest

    permissions:
      contents: write

    steps:
      - name: Checkout the code
        uses: actions/checkout@v4
        with:
          token: ${{ secrets.AUTOMATED_STATUS_CHECKS }}

      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ env.RUST_TOOLCHAIN }}

      - name: Setup Rust cache
        uses: Swatinem/rust-cache@v2

      - run: |
          cargo run -- -c "${{ env.CHECK_NAME }}"
          git config --global user.name 'Your Name'
          git config --global user.email 'your-username@users.noreply.github.com'
          git add assets/ config/
          git commit -m "Update status for ${{ env.CHECK_NAME }}" && git push
```

And then you can set it up where a push to the repository triggers a site redeploy from a cloud provider.

_Disclaimer_: Keep in mind, when you give it the ability to write to the repository using your Personal Access Token it will trigger other actions when it pushes. Formatting, Linting, Deployments etc.

_Note_: As far as I understand, GitHub Actions do not allow for running ping checks. I have not tried to test a port check yet so beware it might not work.

## Cron

### Running all checks at the same interval

Open up your crontab for editing:

```bash
crontab -e
```

Add the following line to run all checks every 5 minutes:

```bash
# Run all checks every 5 minutes
*/5 * * * * /path/to/nanowatchrs --all
```

### Running individual checks at different intervals

Open up your crontab for editing:

```bash
crontab -e
```

Add the following lines to run individual checks at different intervals:

```bash
# Run backend check every minute
* * * * * /path/to/nanowatchrs --check "Backend"

# Run database port check every 5 minutes
*/5 * * * * /path/to/nanowatchrs --check "Database"

# Run backup check every hour
0 * * * * /path/to/nanowatchrs --check "Backup Service"
```

_Disclaimer_: This has not been tested yet. Use at your own risk.

## Systemd

Not implemented yet.

## Docker

With a crontab that is defined like so:

In your `crontab` file:

```bash
# Run all checks every 5 minutes
*/5 * * * * /path/to/nanowatchrs --all >> /var/log/cron.log 2>&1
```

```Dockerfile
FROM ubuntu:latest

# Add crontab file in the cron directory
ADD crontab /etc/cron.d/nanowatchrs-cron

# Give execution rights on the cron job
RUN chmod 0644 /etc/cron.d/nanowatchrs-cron

# Create the log file to be able to run tail
RUN touch /var/log/cron.log

#Install Cron
RUN apt-get update
RUN apt-get -y install cron

# Run the command on container startup
CMD cron && tail -f /var/log/cron.log
```

_Disclaimer_: This has not been tested yet. Use at your own risk.

## Cloudflare

_Note_: Cloudflare workers do not support ping or port checks, only HTTP checks. This is due to the limitations of the runtime environment and not this program itself.

Not implemented yet.

## AWS Lambda

_Note_: AWS Lambda Functions do not support ping checks, only HTTP and port checks. This is due to the limitations of the runtime environment and not this program itself.

Not implemented yet.

# Serving

The `assets/` folder will contain the generated status page files. These files can be served as static assets by any web server. The page is designed to be served as a static site, and does not require any server side processing.

There are future plans to be able to specify a different output directory, or even an S3 compatible bucket, but they are not implemented yet.

# Backup

Since it's just static assets, I recommend tracking the changes to the files (history, config, assets, ...) in a version control system like git, and commit the changes regularliy.

I am exploring the ability to run the cron jobs in GitHub actions and commiting the changes back to the repository, but that is not implemented yet.
