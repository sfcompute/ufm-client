//! TLS verification tests against a local TLS server.

use std::sync::Arc;

use rcgen::{BasicConstraints, CertificateParams, DnType, IsCa, KeyPair, SanType};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use ufm_client::{Auth, Error, UfmClient};

const VERSION_BODY: &str = r#"{"opensm":"5.20.0","ufm_release_version":"6.18.0-5"}"#;

struct TestCa {
    cert: rcgen::Certificate,
    key: KeyPair,
}

fn new_ca(name: &str) -> TestCa {
    let mut params = CertificateParams::default();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.distinguished_name.push(DnType::CommonName, name);
    let key = KeyPair::generate().unwrap();
    let cert = params.self_signed(&key).unwrap();
    TestCa { cert, key }
}

/// Serves one canned version response over TLS with a leaf for 127.0.0.1 signed by `ca`.
async fn serve_version(ca: &TestCa) -> u16 {
    let mut params = CertificateParams::default();
    params.subject_alt_names = vec![SanType::IpAddress("127.0.0.1".parse().unwrap())];
    let leaf_key = KeyPair::generate().unwrap();
    let leaf = params.signed_by(&leaf_key, &ca.cert, &ca.key).unwrap();
    let config = ServerConfig::builder_with_provider(Arc::new(
        tokio_rustls::rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(
        vec![leaf.der().clone()],
        PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(leaf_key.serialize_der())),
    )
    .unwrap();
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        loop {
            let (tcp, _) = listener.accept().await.unwrap();
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                let Ok(mut tls) = acceptor.accept(tcp).await else {
                    return;
                };
                let mut buf = [0u8; 4096];
                let _ = tls.read(&mut buf).await;
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{VERSION_BODY}",
                    VERSION_BODY.len()
                );
                let _ = tls.write_all(response.as_bytes()).await;
                let _ = tls.shutdown().await;
            });
        }
    });
    port
}

fn client_trusting(port: u16, root_pem: String) -> UfmClient {
    UfmClient::builder()
        .base_url(format!("https://127.0.0.1:{port}"))
        .auth(Auth::basic("admin", "password"))
        .max_retries(0)
        .root_certificates_pem(root_pem)
        .build()
        .unwrap()
}

#[tokio::test]
async fn trusts_server_signed_by_configured_root() {
    let ca = new_ca("test root");
    let port = serve_version(&ca).await;
    let version = client_trusting(port, ca.cert.pem())
        .version()
        .await
        .unwrap();
    assert_eq!(version.ufm_release_version, "6.18.0-5");
}

#[tokio::test]
async fn rejects_server_signed_by_unrelated_root() {
    let server_ca = new_ca("server root");
    let other_ca = new_ca("unrelated root");
    let port = serve_version(&server_ca).await;
    let err = client_trusting(port, other_ca.cert.pem())
        .version()
        .await
        .unwrap_err();
    match err {
        Error::Http(e) => assert!(e.is_connect(), "expected connect error, got {e:?}"),
        other => panic!("expected TLS connect failure, got {other:?}"),
    }
}

#[test]
fn roots_with_verify_ssl_false_is_rejected() {
    let ca = new_ca("test root");
    let err = UfmClient::builder()
        .base_url("https://127.0.0.1")
        .auth(Auth::basic("admin", "password"))
        .root_certificates_pem(ca.cert.pem())
        .verify_ssl(false)
        .build()
        .unwrap_err();
    assert!(matches!(err, Error::Configuration { .. }), "{err:?}");
}

#[test]
fn garbage_root_pem_is_rejected() {
    let err = UfmClient::builder()
        .base_url("https://127.0.0.1")
        .auth(Auth::basic("admin", "password"))
        .root_certificates_pem("not a certificate")
        .build()
        .unwrap_err();
    assert!(matches!(err, Error::Configuration { .. }), "{err:?}");
}
