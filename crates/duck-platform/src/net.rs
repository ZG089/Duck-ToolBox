//! Shared HTTP client.
//!
//! Android's system trust store is not readable through rustls, so on device the client
//! ships Mozilla's root set (`webpki-roots`) instead.

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use rustls::{ClientConfig, RootCertStore};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

pub fn http_client() -> Result<Client> {
    let builder = Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .user_agent(concat!("duck-toolbox/", env!("CARGO_PKG_VERSION")));

    let builder = if cfg!(target_os = "android") {
        builder.tls_backend_preconfigured(embedded_rustls_config())
    } else {
        builder
    };

    builder.build().context("build HTTP client")
}

pub fn embedded_rustls_config() -> ClientConfig {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let mut config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    config
}

#[cfg(test)]
mod tests {
    use super::embedded_rustls_config;

    #[test]
    fn embedded_config_offers_h2_and_http11() {
        assert_eq!(
            embedded_rustls_config().alpn_protocols,
            vec![b"h2".to_vec(), b"http/1.1".to_vec()]
        );
    }
}
