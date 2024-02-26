use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
#[allow(dead_code)]
pub enum Error {
    #[error("invalid CA")]
    Tls(#[from] rcgen::Error),
    #[error("network error")]
    Network(#[from] hyper::Error),
    #[error("unable to decode body")]
    Decode,
    #[error("unknown error")]
    Unknown,
}
