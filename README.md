# servy

> A simple HTTP file server with some basic URL shortening/redirect functionality

This project is a reimplementation of my [`gosherve`](https://github.com/jnsgruk/gosherve) project in Rust. I used this project as a learning exercise.

Like `gosherve`, `servy` can:

- Serve files from a specified directory
- Serve redirects specified in a file hosted at a URL
- Report some metrics about the redirects served

You can read more about the original goals in the [`gosherve`](https://github.com/jnsgruk/gosherve) repo.

## Configuration

The server is configured with two environment variables:

| Variable Name         |   Type   | Notes                                                                            |
| :-------------------- | :------: | :------------------------------------------------------------------------------- |
| `SERVY_ASSETS_DIR`    | `string` | Path to directory containing web assets to be packed into the binary.            |
| `SERVY_REDIRECTS_URL` | `string` | URL containing a list of aliases and corresponding redirect URLs                 |
| `SERVY_LOG_LEVEL`     | `string` | Sets the log level. One of: `info`, `debug`, `warn`, `error`. Default is `info`. |
| `SERVY_HOST`          | `string` | The server's bind address. Default is `127.0.0.1`                                |
| `SERVY_PORT`          | `string` | The server's port. Default is `8080`                                             |
| `SERVY_METRICS_PORT`  | `string` | The server's metrics endpoint port. Default is `8081`                            |

## Hacking

The application has minimal dependencies and can be run like so:

```bash
git clone https://github.com/jnsgruk/servy
cd servy

# Export some variables to configure gosherve
export SERVY_REDIRECTS_URL="https://gist.githubusercontent.com/someuser/somegisthash/raw"
export SERVY_ASSETS_DIR="/path/to/some/files"

# Run it!
cargo run
```

### Running the tests

The tests are written on the assumption that a fake site is served, and thus the `SERVY_ASSETS_DIR` variable must be set to `<REPO>/tests/servy_assets` before running tests. This is taken care of automatically if you're using nix:

```bash
git clone https://github.com/jnsgruk/servy
cd servy

# With nix
nix develop
cargo test

# Without nix
export SERVY_ASSETS_DIR="$PWD/tests/servy_assets"
cargo test
```

## Build my website!

This project was designed to be a 1:1 replacement for `gosherve` and the build of my website that's available [on Github](https://github.com/jnsgruk).

Thus, a build target is provided for both a `jnsgruk` binary, and a Docker container that's the equivalent of what's deployed on Fly.io for my production website:

```bash
# Run my website locally
nix run .#jnsgruk

# Build/load a Docker container with the binary above
nix build .#jnsgruk-container
docker load < result
```
