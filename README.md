# NanoWatchrs

This is a super small and simple status page. It is meant to be lean when served, and lean when ran. It uses Rust to poll the status of the services, and then creates static HTML files that can be served as static assets (who woulda thunk).

Very minimal CSS and JS is used to make the page look and behave as necessary, but JS is not required for the page to function.

This is meant to be launched as a cron job, or as a repeated task, or something similar, to keep the status page up to date. Rust is used to allow for running as a native binary, or inside a wasm runtime.

# Configuration

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
      "header": "Status Page Header"
    }
  }
}
```

The `"site"` settings represent general settings. These are used in the meta tags in the header of the page.

- `name`: Your site or service name
- `description`: Brief description of your service
- `url`: Main website URL
- `logo`: URL to your logo image

the `"page"` settings represent the settings for the page itself. These are used in the title and header of the page.

- `title`: Browser tab title
- `header`: Main heading displayed on the status page

## Service Checks

The `checks` array defines the services to monitor. Three types of checks are supported:

- `name`: Display name for the service. Important as it's used as the identifier
- `description`: (Optional) Service description. Displayed under the name on the status section
- `type`: Check type (`http`, `ping`, or `port`)
- `target`: URL, hostname, or IP to check
- `page_link`: (Optional) URL to service documentation or information
- `expected_status`: (HTTP only) Expected response code
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

_Note_: Ping checks might not be supported on some setups (like cloudflare workers).

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

# Running

There are many different ways to run the status page job. It is not meant to be a long running service, but instead relies on external programs to call it at regular intervals. The intention was to use cron scheduling that is supported on most systems, and in most cloud providers with serverless functions.

The way to specify different frequencies for each individual check is not to have them in a configuration file. Instead, you are meant to specify which check to perform, and use the frequency of the calling program to determine how often to run the checks.

Checks are identified by their `"name"` field in the configuration files `"checks"` section, and are specified by setting the `--check` or `-c` flag when running the program. If you want all checks to be run, you can use the `--all` or `-a` flag.

## Cron

### Running all checks at the same interval

Open up your crontab for editing:

```sh
crontab -e
```

Add the following line to run all checks every 5 minutes:

```sh
# Run all checks every 5 minutes
*/5 * * * * /path/to/nanowatchrs --all
```

### Running individual checks at different intervals

Open up your crontab for editing:

```sh
crontab -e
```

Add the following lines to run individual checks at different intervals:

```sh
# Run backend check every minute
* * * * * /path/to/nanowatchrs --check "Backend"

# Run database port check every 5 minutes
*/5 * * * * /path/to/nanowatchrs --check "Database"

# Run backup check every hour
0 * * * * /path/to/nanowatchrs --check "Backup Service"
```

## Systemd

Not implemented yet.

## Docker

With a crontab that is defined like so:

In your `crontab` file:

```sh
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
