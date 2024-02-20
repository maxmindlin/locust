mod ca;
mod error;
mod rewind;
mod service;

use ca::RcgenAuthority;
use futures::Future;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Server,
};
use locust_core::new_pool;
use rustls_pemfile as pemfile;
use sqlx::PgPool;
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use tracing::*;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}

struct ServiceWrapper {
    ca: Arc<RcgenAuthority>,
    db: Arc<PgPool>,
}

impl ServiceWrapper {
    pub async fn start<F: Future<Output = ()>>(
        self,
        shutdown_signal: F,
    ) -> Result<(), error::Error> {
        let make_service = make_service_fn(move |_conn: &AddrStream| {
            let ca = Arc::clone(&self.ca);
            let db = Arc::clone(&self.db);
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    service::Service::new(Arc::clone(&ca), Arc::clone(&db)).proxy(req)
                }))
            }
        });

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        Server::try_bind(&addr)?
            .http1_preserve_header_case(true)
            .http1_title_case_headers(true)
            .serve(make_service)
            .with_graceful_shutdown(shutdown_signal)
            .await
            .map_err(Into::into)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let mut private_key_bytes: &[u8] = include_bytes!("ca/locust.key");
    let mut ca_cert_bytes: &[u8] = include_bytes!("ca/locust.cer");
    let private_key = tokio_rustls::rustls::PrivateKey(
        pemfile::pkcs8_private_keys(&mut private_key_bytes)
            .next()
            .unwrap()
            .expect("Failed to parse private key")
            .secret_pkcs8_der()
            .to_vec(),
    );
    let ca_cert = tokio_rustls::rustls::Certificate(
        pemfile::certs(&mut ca_cert_bytes)
            .next()
            .unwrap()
            .expect("Failed to parse CA certificate")
            .to_vec(),
    );

    let ca_auth = ca::RcgenAuthority::new(private_key, ca_cert, 1_000)
        .expect("Failed to create Certificate Authority");
    let db_pool = new_pool().await.expect("Error creating db pool");

    let wrapper = ServiceWrapper {
        ca: Arc::new(ca_auth),
        db: Arc::new(db_pool),
    };

    println!("Starting up proxy server!");
    if let Err(e) = wrapper.start(shutdown_signal()).await {
        error!("{}", e);
    }
}
