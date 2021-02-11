# rev-proxy

![CI](https://github.com/swift-nav/rev-proxy-rs/workflows/CI/badge.svg)

## Usage

```
rev-proxy 1.1.0 (83e860c)
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

REV_PROXY_SHUTDOWN_KEY   - a key that must be presented to the
                           upstream server to initiate
                           a shutdown, e.g. `2a2a3a6dafe30...`

REV_PROXY_SHUTDOWN_URL   - the URL to invoke when a shutdown is
                           triggered, the value from
                           `REV_PROXY_SHUTDOWN_KEY` is appended
                           to this URL, e.g.
                           `http://127.0.0.1:8080/shutdown?key=`
```

## Copyright

```
Copyright (C) 2020 Swift Navigation Inc.
Contact: Swift Navigation <dev@swiftnav.com>

This source is subject to the license found in the file 'LICENSE' which must be
be distributed together with this source. All other rights reserved.

THIS CODE AND INFORMATION IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND,
EITHER EXPRESSED OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND/OR FITNESS FOR A PARTICULAR PURPOSE.
```
