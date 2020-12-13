use imap::{self, Session};
use native_tls::TlsStream;
use serde::Deserialize;
use std::net::TcpStream;

#[derive(Debug, Deserialize)]
pub struct ImapConfig {
    pub tz_offset: i64,
    pub domain: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub fn imap_session(config: &ImapConfig) -> Session<TlsStream<TcpStream>> {
    let tls = native_tls::TlsConnector::builder().build().unwrap();
    let client = imap::connect(
        (config.domain.as_str(), config.port),
        config.domain.as_str(),
        &tls,
    )
    .unwrap();
    client
        .login(&config.username, &config.password)
        .map_err(|e| e.0)
        .unwrap()
}
