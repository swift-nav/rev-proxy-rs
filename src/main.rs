use std::boxed::Box;
use std::error::Error;
use std::net::SocketAddr;

use serde::Deserialize;
use tokio::sync::oneshot;
use warp::Filter;

use warp_reverse_proxy::reverse_proxy_filter;

#[derive(Deserialize, Debug, Clone)]
struct Config {
    rev_proxy_listen_address: String,
    rev_proxy_base_path: String,
    rev_proxy_upstream_url: String,
    rev_proxy_shutdown_key: String,
    rev_proxy_upstream_shutdown_url: String,
}

#[derive(Deserialize)]
struct Params {
    key: String,
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type StdResult<T, E> = std::result::Result<T, E>;

use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::cell::Cell;

type SenderT = oneshot::Sender<bool>;
type MaybeSenderT = Option<SenderT>;
type ArcSenderT = Arc<Mutex<Cell<MaybeSenderT>>>;

fn unwrap_shutdown_tx<'a>(tx: ArcSenderT) -> MaybeSenderT {
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

use warp::{hyper::body::Bytes, Rejection, Reply};

async fn log_response(response: warp::http::Response<Bytes>) -> StdResult<impl Reply, Rejection> {
    log::debug!("{:?}", response);
    Ok(response)
}

use clap::{Arg, App, SubCommand};

#[tokio::main]
async fn main() -> Result<()> {

    let matches = App::new("rev-proxy")
        .version(env!("VERGEN_SEMVER_LIGHTWEIGHT"))
        .author("Swift Navigation <dev@swift-nav.com>")
        .about("Reverse proxy middleware to handle clean shutdowns")
        .get_matches();

    println!("{:#?}", matches);

    env_logger::init();
    let config = envy::from_env::<Config>()?;

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
        config.rev_proxy_upstream_shutdown_url, config.rev_proxy_shutdown_key
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
