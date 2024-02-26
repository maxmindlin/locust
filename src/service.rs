use crate::{ca::CertificateAuthority, rewind::Rewind, worker::DBJob};

use cookie::Cookie;
use http::{
    header::COOKIE,
    uri::{Authority, Scheme},
    HeaderValue,
};
use hyper::{
    client::HttpConnector, header::Entry, server::conn::Http, service::service_fn, Body, Client,
    Method, Request, Response, StatusCode, Uri,
};
use hyper_proxy::{Intercept, Proxy, ProxyConnector};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use locust_core::{
    crud::{
        self,
        proxies::{
            create_proxy_session, get_general_proxy, get_proxy_by_domain, get_proxy_by_id,
            get_proxy_session,
        },
    },
    models,
};
use sqlx::PgPool;
use std::{
    convert::Infallible,
    future::Future,
    sync::{mpsc, Arc},
    time::Instant,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite},
    net::TcpStream,
    task::JoinHandle,
};
use tokio_rustls::TlsAcceptor;
use tracing::{error, info_span, warn, Instrument, Span};

const SESSION_KEY: &str = "locust_session";

fn bad_request() -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::empty())
        .expect("Failed to build response")
}

fn spawn_with_trace<T: Send + Sync + 'static>(
    fut: impl Future<Output = T> + Send + 'static,
    span: Span,
) -> JoinHandle<T> {
    tokio::spawn(fut.instrument(span))
}

pub struct Service<CA> {
    ca: Arc<CA>,
    db: Arc<PgPool>,
    db_job_chan: mpsc::Sender<DBJob>,
}

impl<CA> Clone for Service<CA> {
    fn clone(&self) -> Self {
        Self {
            ca: Arc::clone(&self.ca),
            db: Arc::clone(&self.db),
            db_job_chan: self.db_job_chan.clone(),
        }
    }
}

impl<CA> Service<CA>
where
    CA: CertificateAuthority,
{
    pub fn new(ca: Arc<CA>, db: Arc<PgPool>, db_job_chan: mpsc::Sender<DBJob>) -> Self {
        Self {
            ca,
            db,
            db_job_chan,
        }
    }

    pub async fn proxy(self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        println!("REQUEST: {req:?}");
        if req.method() == Method::CONNECT {
            Ok(self.process_connect(req))
        } else if hyper_tungstenite::is_upgrade_request(&req) {
            unimplemented!()
        } else {
            let maybe_session = extract_session_cookie(&req);
            let host: Option<String> = req.uri().host().map(Into::into);
            let (upstream_proxy, session_id) = match maybe_session {
                None => {
                    let proxy = self
                        .get_upstream_proxy(host.clone())
                        .await
                        .expect("Error getting proxy for client");
                    println!("CREATING SESSION");
                    let session = create_proxy_session(&self.db, proxy.id)
                        .await
                        .expect("Error creation proxy session");
                    (proxy, session.id)
                }
                Some(id) => {
                    println!("USING SESSION");
                    let session = get_proxy_session(&self.db, id)
                        .await
                        .expect("Error getting proxy session");
                    let proxy = get_proxy_by_id(&self.db, session.proxy_id)
                        .await
                        .expect("Error getting proxy from session");
                    (proxy, session.id)
                }
            };
            let client = build_client(&upstream_proxy);
            let start_time = Instant::now();
            println!("SENDING REQ");
            let mut res = client
                .request(normalize_request(req))
                .await
                .expect("Error with request");
            let duration = start_time.elapsed().as_millis();
            println!("RESPONSE: {res:?}");
            if let Err(e) = self.db_job_chan.send(DBJob::ProxyResponse {
                proxy_id: upstream_proxy.id,
                status: res.status(),
                response_time: duration as u32,
                domain: host.map(Into::into),
            }) {
                println!("Error sending proxy response job: {e}");
            }
            res.headers_mut().insert(
                "Set-Cookie",
                HeaderValue::from_str(format!("{SESSION_KEY}={session_id}").as_ref()).unwrap(),
            );
            Ok(res)
        }
    }

    async fn get_upstream_proxy(
        &self,
        host: Option<String>,
    ) -> Result<models::proxies::Proxy, sqlx::Error> {
        match host {
            Some(host) => get_proxy_by_domain(&self.db, &host).await,
            None => get_general_proxy(&self.db).await,
        }
    }

    fn process_connect(self, mut req: Request<Body>) -> Response<Body> {
        match req.uri().authority().cloned() {
            Some(authority) => {
                let span = info_span!("process_connect");
                let fut = async move {
                    match hyper::upgrade::on(&mut req).await {
                        Ok(mut upgraded) => {
                            let mut buffer = [0; 4];
                            let bytes_read = match upgraded.read(&mut buffer).await {
                                Ok(bytes_read) => bytes_read,
                                Err(e) => {
                                    error!("Failed to read from upgraded connection: {}", e);
                                    return;
                                }
                            };

                            let mut upgraded = Rewind::new_buffered(
                                upgraded,
                                bytes::Bytes::copy_from_slice(buffer[..bytes_read].as_ref()),
                            );

                            if buffer == *b"GET " {
                                if let Err(e) =
                                    self.serve_stream(upgraded, Scheme::HTTP, authority).await
                                {
                                    error!("WebSocket connect error: {}", e);
                                }

                                return;
                            } else if buffer[..2] == *b"\x16\x03" {
                                let server_config = self
                                    .ca
                                    .gen_server_config(&authority)
                                    .instrument(info_span!("gen_server_config"))
                                    .await;

                                let stream =
                                    match TlsAcceptor::from(server_config).accept(upgraded).await {
                                        Ok(stream) => stream,
                                        Err(e) => {
                                            error!("Failed to establish TLS connection: {}", e);
                                            return;
                                        }
                                    };

                                if let Err(e) =
                                    self.serve_stream(stream, Scheme::HTTPS, authority).await
                                {
                                    if !e.to_string().starts_with("error shutting down connection")
                                    {
                                        error!("HTTPS connect error: {}", e);
                                    }
                                }

                                return;
                            } else {
                                warn!(
                                    "Unknown protocol, read '{:02X?}' from upgraded connection",
                                    &buffer[..bytes_read]
                                );
                            }

                            let mut server = match TcpStream::connect(authority.as_ref()).await {
                                Ok(server) => server,
                                Err(e) => {
                                    error!("Failed to connect to {}: {}", authority, e);
                                    return;
                                }
                            };

                            if let Err(e) =
                                tokio::io::copy_bidirectional(&mut upgraded, &mut server).await
                            {
                                error!("Failed to tunnel to {}: {}", authority, e);
                            }
                        }
                        Err(e) => error!("Upgrade error: {}", e),
                    };
                };

                spawn_with_trace(fut, span);
                Response::new(Body::empty())
            }
            None => bad_request(),
        }
    }

    async fn serve_stream<I>(
        self,
        stream: I,
        scheme: Scheme,
        authority: Authority,
    ) -> Result<(), hyper::Error>
    where
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let service = service_fn(|mut req| {
            if req.version() == hyper::Version::HTTP_10 || req.version() == hyper::Version::HTTP_11
            {
                let (mut parts, body) = req.into_parts();

                parts.uri = {
                    let mut parts = parts.uri.into_parts();
                    parts.scheme = Some(scheme.clone());
                    parts.authority = Some(authority.clone());
                    Uri::from_parts(parts).expect("Failed to build URI")
                };

                req = Request::from_parts(parts, body);
            };

            self.clone().proxy(req)
        });

        Http::new()
            .serve_connection(stream, service)
            .with_upgrades()
            .await
    }
}

fn build_client(
    upstream_proxy: &models::proxies::Proxy,
) -> Client<ProxyConnector<HttpsConnector<HttpConnector>>> {
    let https = HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_or_http()
        .enable_http1()
        .build();

    let mut proxy = Proxy::new(
        Intercept::All,
        format!(
            "{}://{}:{}",
            upstream_proxy.protocol, upstream_proxy.host, upstream_proxy.port
        )
        .parse()
        .unwrap(),
    );
    if let (Some(usr), Some(pwd)) = (&upstream_proxy.username, &upstream_proxy.password) {
        let auth = headers::Authorization::basic(usr, pwd);
        proxy.set_authorization(auth);
    }
    let connector = ProxyConnector::from_proxy(https, proxy).unwrap();

    Client::builder()
        .http1_title_case_headers(true)
        .http1_preserve_header_case(true)
        .build(connector)
}

fn normalize_request<T>(mut req: Request<T>) -> Request<T> {
    // Hyper will automatically add a Host header if needed.
    req.headers_mut().remove(hyper::header::HOST);

    // HTTP/2 supports multiple cookie headers, but HTTP/1.x only supports one.
    if let Entry::Occupied(mut cookies) = req.headers_mut().entry(hyper::header::COOKIE) {
        let joined_cookies = bstr::join(b"; ", cookies.iter());
        cookies.insert(joined_cookies.try_into().expect("Failed to join cookies"));
    }

    *req.version_mut() = hyper::Version::HTTP_11;
    req
}

fn extract_session_cookie<T>(req: &Request<T>) -> Option<i32> {
    let cookies = req.headers().get(COOKIE)?;
    for cookie in Cookie::split_parse(cookies.to_str().unwrap()) {
        let cookie = cookie.unwrap();
        if cookie.name() == SESSION_KEY {
            let val = cookie.value();
            return Some(val.parse().unwrap());
        }
    }

    None
}
