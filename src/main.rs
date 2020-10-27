use std::boxed::Box;
use std::cell::Cell;
use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use clap::App;
use indoc::indoc;
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

#[derive(Deserialize)]
struct Params {
    key: String,
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type StdResult<T, E> = std::result::Result<T, E>;

type SenderT = oneshot::Sender<bool>;
type MaybeSenderT = Option<SenderT>;
type ArcSenderT = Arc<Mutex<Cell<MaybeSenderT>>>;

fn unwrap_shutdown_tx(tx: ArcSenderT) -> MaybeSenderT {
    tx.lock().ok()?.take()
}

fn with_shutdown_tx(
    tx_input: SenderT,
) -> impl Filter<Extract = (ArcSenderT,), Error = Infallible> + Clone {
    let tx = Arc::new(Mutex::new(Cell::new(Some(tx_input))));
    warp::any().map(move || tx.clone())
}

fn with_config(config: Config) -> impl Filter<Extract = (Config,), Error = Infallible> + Clone {
    warp::any().map(move || config.clone())
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

            REV_PROXY_SHUTDOWN_KEY   - a key that must be matched to trigger
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

    let shutdown_route = warp::path!("shutdown")
        .and(warp::query::<Params>())
        .and(with_shutdown_tx(tx))
        .and(with_config(config.clone()))
        .and_then(
            |params: Params, tx: ArcSenderT, config: Config| async move {
                if params.key != config.rev_proxy_shutdown_key {
                    Err(warp::reject::not_found())
                } else {
                    unwrap_shutdown_tx(tx)
                        .ok_or("failed to unwrap shutdown signaler")
                        .map(|tx: SenderT| {
                            log::info!("sending shutdown signal");
                            tx.send(true)
                        })
                        .and(Ok("success"))
                        .or_else(|e| {
                            log::error!("sending shutdown signal failed: {:?}", e);
                            Ok("failure")
                        })
                }
            },
        );

    let upstream_url = config.rev_proxy_upstream_url;
    let base_path = config.rev_proxy_base_path;

    let log = warp::log("rev_proxy");

    let app = shutdown_route
        .or(warp::any().and(reverse_proxy_filter(base_path, upstream_url).and_then(log_response)))
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
