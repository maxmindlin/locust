<h1 align="center">
<img src="https://raw.githubusercontent.com/maxmindlin/locust/main/assets/logo.png" width="200">
<img alt="GitHub License" src="https://img.shields.io/github/license/maxmindlin/locust?style=for-the-badge">
</h1><br>

# Locust

Locust is a MITM proxy that routes traffic to various upstream proxies. Locust comes with a CLI that facilitates proxy configuration, importing, querying, and creating custom Squid proxies that are automatically used by the proxy server.

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
