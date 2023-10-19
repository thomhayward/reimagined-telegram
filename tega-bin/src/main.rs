use clap::Parser;
use pem::Pem;
use rustls::{
	client::{ServerCertVerified, ServerCertVerifier},
	Certificate, RootCertStore,
};
use std::{
	fs::File,
	io::{self, BufReader},
	net::IpAddr,
	path::{Path, PathBuf},
	sync::{Arc, Mutex},
};
use tega_client::Client;
use tokio::sync::oneshot;

#[derive(Parser)]
#[clap(
	name = "teg",
	about = "A command-line client for the Tesla Backup Gateway 2.",
	version
)]
struct Arguments {
	/// IP Address of the Tesla Backup Gateway 2.
	ip_address: IpAddr,

	/// Path to a PEM file containing the certificate for the Tesla Backup
	/// Gateway 2.
	certificate: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
	let arguments = Arguments::parse();
	let certs = load_certificates_from_pem(&arguments.certificate)?;

	let client = Client::new((arguments.ip_address, 443).try_into()?, certs)?;
	let status = client.status().await?;
	println!("{status:#?}");

	let certificate = fetch_certificate(arguments.ip_address).await?;
	let p = Pem::new("CERTIFICATE", certificate.0);
	let pem = pem::encode(&p);

	println!("{pem}");

	Ok(())
}

fn load_certificates_from_pem(path: &Path) -> io::Result<Vec<Certificate>> {
	let file = File::open(path)?;
	let mut reader = BufReader::new(file);
	let certs = rustls_pemfile::certs(&mut reader)?;

	Ok(certs.into_iter().map(Certificate).collect())
}

/// Downloads the self-signed certificate from the Tesla Backup Gateway 2 with
/// the specified IP address.
pub async fn fetch_certificate(addr: IpAddr) -> reqwest::Result<Certificate> {
	const BASE_NAME: &str = "teg";

	struct FetchCertificateVerifier(Mutex<Option<oneshot::Sender<Certificate>>>);

	impl ServerCertVerifier for FetchCertificateVerifier {
		fn verify_server_cert(
			&self,
			end_entity: &Certificate,
			_intermediates: &[Certificate],
			_server_name: &rustls::ServerName,
			_scts: &mut dyn Iterator<Item = &[u8]>,
			_ocsp_response: &[u8],
			_now: std::time::SystemTime,
		) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
			let cert = webpki::EndEntityCert::try_from(end_entity.0.as_ref()).map_err(|_| {
				rustls::Error::InvalidCertificate(rustls::CertificateError::BadEncoding)
			})?;

			let _ = cert
				.verify_is_valid_for_dns_name(
					webpki::DnsNameRef::try_from_ascii_str(BASE_NAME).unwrap(),
				)
				.map_err(|_| {
					rustls::Error::InvalidCertificate(rustls::CertificateError::NotValidForName)
				});

			let mut sender = self.0.lock().unwrap();
			if let Some(sender) = sender.take() {
				let _ = sender.send(end_entity.clone());
			};

			Ok(ServerCertVerified::assertion())
		}
	}

	let (tx, rx) = oneshot::channel();

	let mut config = rustls::ClientConfig::builder()
		.with_safe_defaults()
		.with_root_certificates(RootCertStore::empty())
		.with_no_client_auth();

	// Install our custom certificate verifier.
	config
		.dangerous()
		.set_certificate_verifier(Arc::new(FetchCertificateVerifier(Mutex::new(Some(tx)))));

	let client = reqwest::Client::builder()
		.use_preconfigured_tls(config)
		.resolve(BASE_NAME, (addr, 443).into())
		.build()?;

	// Make a request to fetch the server's certificate.
	let request = client.get(format!("https://{BASE_NAME}/api/status")).send();
	tokio::pin!(request);
	tokio::pin!(rx);

	let certificate = loop {
		tokio::select! {
			response = &mut request => {
				let _ = response?;
				println!("received response");
				continue;
			}
			Ok(certificate) = &mut rx => {
				break certificate;
			}
		}
	};

	Ok(certificate)
}
