<h1 align="center">
<img src="https://raw.githubusercontent.com/maxmindlin/locust/main/assets/logo.png" width="200"><br>
<img alt="GitHub License" src="https://img.shields.io/github/license/maxmindlin/locust?style=for-the-badge">
<a href="https://github.com/maxmindlin/locust/releases/latest" target="blank">
  <img alt="GitHub Release" src="https://img.shields.io/github/v/release/maxmindlin/locust?style=for-the-badge">
</a>
<img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/maxmindlin/locust/test.yml?style=for-the-badge&label=CI">
</h1>

# Locust

Locust is a MITM proxy that routes traffic to various upstream proxies. Locust comes with a CLI that facilitates proxy configuration, importing, querying, and creating custom Squid proxies that are automatically used by the proxy server.

## Features

- Proxy server that smartly routes traffic to an appropriate upstream proxy.
  - Easily configurable via a tagging system.
- CLI that makes managing proxies easy.
  - File importing that works out of the box with known proxy providers.
  - Squid proxy farm commands that allow for the creation, deletion, and cycling of squid VMs that are automatically picked up and used by the server (currently only works with GCP).
  - Configuration commands that make sure your proxies work out of the box.
  - Querying via tags that gives visibility into proxies without having to execute SQL.

## Requirements

### CLI

1. `gloud` CLI installed and authenticated.
2. using the `farm` command will require sufficient GCP privileges for VM creation and destruction.
3. ENV vars set with the proper PostgreSQL connection parameters. See `.env` file for default settings.

### Proxy server

The proxy server currently runs as a dockerfile. A compose file is provided as well for easy spinup. Without the compose you will need ENV vars set with the proper PostgreSQL connection parameters. See `.env` file for the default settings.

## Running

### CLI

- `cargo run -p locust-cli -h`

### Proxy server

1. `export $(cat .env)`
2. `docker-compose up --build`
3. `cargo run`

TODO: include proxy server in the compose file.
