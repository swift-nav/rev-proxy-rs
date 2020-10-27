# rev-proxy

```
rev-proxy 1.0.0 (3dc2113)
Swift Navigation <dev@swift-nav.com>
Reverse proxy middleware to handle clean shutdowns

USAGE:
    rev-proxy

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

Requires the following environment variables for configuration:

REV_PROXY_LISTEN_ADDRESS - the listen address for the service,
                           e.g. `127.0.0.1:8008`

REV_PROXY_BASE_PATH      - the base path to be included in
                           requests to the upstream proxy,
                           e.g. `/upstream/path`

REV_PROXY_UPSTREAM_URL   - the URL of the upstream server,
                           e.g. `http://127.0.0.1:8080/`

REV_PROXY_SHUTDOWN_KEY   - a key that must be matched to trigger
                           a shutdown, e.g. `2a2a3a6dafe30...`

REV_PROXY_SHUTDOWN_URL   - the URL to invoke when a shutdown is
                           triggered, the value from
                           `REV_PROXY_SHUTDOWN_KEY` is appended
                           to this URL, e.g.
                           `http://127.0.0.1:8080/shutdown?key=`
```
