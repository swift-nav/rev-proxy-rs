use std::boxed::Box;
use std::cell::Cell;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Mutex;

use clap::App;
use indoc::indoc;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use tokio::sync::oneshot;
use warp::{hyper::body::Bytes, Filter, Rejection, Reply};
use warp_reverse_proxy::reverse_proxy_filter;

#[derive(Deserialize, Debug, Clone)]
struct Config {
    rev_proxy_listen_address: String,
    rev_proxy_base_path: String,
    rev_proxy_upstream_url: String,
    rev_proxy_shutdown_key: String,
    rev_proxy_shutdown_url: String,
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type StdResult<T, E> = std::result::Result<T, E>;

type SenderT = oneshot::Sender<bool>;
type MaybeSenderT = Option<SenderT>;
type SharedSenderT = Mutex<Cell<MaybeSenderT>>;

static SHUTDOWN_TX: OnceCell<SharedSenderT> = OnceCell::new();

fn setup_termination_handler() {
    ctrlc::set_handler(move || {
        let _: StdResult<_, _> = (|| {
            SHUTDOWN_TX
                .get()
                .ok_or_else(|| {
                    log::error!("shutdown signaler not initialized");
                })?
                .lock()
                .map_err(|_| {
                    log::error!("failed to lock shutdown signaler");
                })?
                .take()
                .ok_or_else(|| {
                    log::error!("termination handler already triggered");
                })
                .map(|tx| tx.send(true))
                .map_err(|_| {
                    log::error!("error triggering termination handler");
                })
        })();
    })
    .expect("error setting SIGINT/SIGTERM handler");
}

async fn log_response(response: warp::http::Response<Bytes>) -> StdResult<impl Reply, Rejection> {
    log::debug!("{:?}", response);
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<()> {
    let version = format!("{} ({})", env!("VERGEN_SEMVER"), env!("VERGEN_SHA_SHORT"));

    let matches = App::new("rev-proxy")
        .version(&*version)
        .author("Swift Navigation <dev@swift-nav.com>")
        .about("Reverse proxy middleware to handle clean shutdowns")
        .after_help(indoc! {"
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
            "})
        .get_matches();

    env_logger::init();
    let config = envy::from_env::<Config>()?;

    log::debug!("{:#?}", matches);

    let (tx, rx) = oneshot::channel();

    SHUTDOWN_TX
        .set(Mutex::new(Cell::new(Some(tx))))
        .ok()
        .expect("failed to set termination signaler");

	setup_termination_handler();

    let upstream_url = config.rev_proxy_upstream_url;
    let base_path = config.rev_proxy_base_path;

    let log = warp::log("rev_proxy");

    let app = reverse_proxy_filter(base_path, upstream_url)
        .and_then(log_response)
        .with(log);

    let listen_addr: SocketAddr = config.rev_proxy_listen_address.parse()?;
    let (addr, server) = warp::serve(app).bind_with_graceful_shutdown(listen_addr, async {
        rx.await.ok();
        log::debug!("got shutdown signal, waiting for all open connections to complete...");
    });

    log::info!("listening on: {}...", addr);
    let _ = tokio::task::spawn(server).await;

    log::info!("shutting down...");

    let shutdown_url = format!(
        "{}{}",
        config.rev_proxy_shutdown_url, config.rev_proxy_shutdown_key
    );

    if shutdown_url.is_empty() {
        log::info!("not issuing upstream shutdown request");
    } else {
        log::info!("issuing upstream shutdown request: {}", shutdown_url);
        let shutdown_resp = reqwest::get(&shutdown_url).await?.text().await?;
        log::info!("upstream shutdown response: {}", shutdown_resp);
    }

    Ok(())
}
