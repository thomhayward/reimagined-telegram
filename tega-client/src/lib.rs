use reqwest::{
	header::{HeaderMap, HeaderValue, ACCEPT},
	ClientBuilder, StatusCode, Url,
};
use rustls::{
	client::{ServerCertVerified, ServerCertVerifier},
	Certificate, RootCertStore,
};
use serde::Serialize;
use std::{fmt, sync::Arc};
use tega_types::{legal::Radio, login::LoginBasic, meters::MetersAggregates, status::Status};

// These need to match.
const BASE_NAME: &str = "teg";
const BASE_URL: &str = "https://teg";

#[derive(Clone, Debug)]
pub struct Client {
	base: Url,
	inner_client: reqwest::Client,
}

#[derive(Debug)]
struct ServerCertificateNotPermitted;

impl fmt::Display for ServerCertificateNotPermitted {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{self:?}")
	}
}

impl std::error::Error for ServerCertificateNotPermitted {}

struct PowerwallVerifier(Vec<Certificate>);

impl ServerCertVerifier for PowerwallVerifier {
	fn verify_server_cert(
		&self,
		end_entity: &Certificate,
		_intermediates: &[Certificate],
		_server_name: &rustls::ServerName,
		_scts: &mut dyn Iterator<Item = &[u8]>,
		_ocsp_response: &[u8],
		_now: std::time::SystemTime,
	) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
		if self.0.iter().any(|cert| cert == end_entity) {
			Ok(ServerCertVerified::assertion())
		} else {
			Err(rustls::Error::InvalidCertificate(
				rustls::CertificateError::Other(Arc::new(ServerCertificateNotPermitted)),
			))
		}
	}
}

impl Client {
	pub fn new(
		addr: std::net::SocketAddr,
		certificates: Vec<Certificate>,
	) -> Result<Self, reqwest::Error> {
		let base = Url::parse(BASE_URL).expect("BASE_URL must be valid");

		let mut config = rustls::ClientConfig::builder()
			.with_safe_defaults()
			.with_root_certificates(RootCertStore::empty())
			.with_no_client_auth();

		// Install our custom certificate verifier.
		config
			.dangerous()
			.set_certificate_verifier(Arc::new(PowerwallVerifier(certificates)));

		// Nearly all of the Tesla Backup Gateway's endpoints returns JSON.
		let mut default_headers = HeaderMap::new();
		default_headers.insert(
			ACCEPT,
			HeaderValue::from_static("application/json; charset=utf-8"),
		);

		let inner_client = ClientBuilder::new()
			.use_preconfigured_tls(config)
			.resolve(BASE_NAME, addr)
			.default_headers(default_headers)
			.build()?;

		Ok(Self { base, inner_client })
	}

	pub async fn login(
		&self,
		username: impl AsRef<str>,
		password: impl AsRef<str>,
	) -> Result<LoginBasic, reqwest::Error> {
		#[derive(Serialize)]
		struct RequestBody<'a> {
			#[serde(borrow)]
			username: &'a str,
			#[serde(borrow)]
			password: &'a str,
		}

		let url = self.base.join("/api/login/Basic").unwrap();

		let response = self
			.inner_client
			.post(url)
			.json(&RequestBody {
				username: username.as_ref(),
				password: password.as_ref(),
			})
			.send()
			.await?;

		assert_eq!(response.status(), StatusCode::OK);
		let body = response.json().await?;

		Ok(body)
	}

	/// Fetches the `/api/meters/aggregates` endpoint from the gateway.
	pub async fn meters_aggregates(&self) -> Result<MetersAggregates, reqwest::Error> {
		let url = self.base.join("/api/meters/aggregates").unwrap();

		let response = self.inner_client.get(url).send().await?;

		assert_eq!(response.status(), StatusCode::OK);
		let body = response.json().await?;

		Ok(body)
	}

	/// Fetches the `/api/status` endpoint from the gateway.
	///
	/// This endpoint does not require authentication.
	pub async fn status(&self) -> Result<Status, reqwest::Error> {
		let url = self.base.join("/api/status").unwrap();

		let response = self.inner_client.get(url).send().await?;

		assert_eq!(response.status(), StatusCode::OK);
		let body = response.json().await?;

		Ok(body)
	}

	pub async fn legal_radio(&self) -> Result<Vec<Radio>, reqwest::Error> {
		let url = self.base.join("/api/legal/radio").unwrap();

		let response = self.inner_client.get(url).send().await?;

		assert_eq!(response.status(), StatusCode::OK);
		let body = response.json().await?;

		Ok(body)
	}
}
