# NanoWatchrs

This is a super small and simple status page. It is meant to be lean when served, and lean when ran. It uses Rust to poll the status of the services, and then creates static HTML files that can be served as static assets (who woulda thunk).

Very minimal CSS and JS is used to make the page look and behave as necessary, but JS is not required for the page to function.

This is meant to be launched as a cron job, or a systemd service, or something similar, to keep the status page up to date. Rust is used to allow for running as a native binary, or inside a wasm runtime.
