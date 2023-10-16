use clap::Parser;
use rustls::Certificate;
use std::{
	fs::File,
	io::{self, BufReader},
	net::IpAddr,
	path::{Path, PathBuf},
};
use tega_client::Client;

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

	Ok(())
}

fn load_certificates_from_pem(path: &Path) -> io::Result<Vec<Certificate>> {
	let file = File::open(path)?;
	let mut reader = BufReader::new(file);
	let certs = rustls_pemfile::certs(&mut reader)?;

	Ok(certs.into_iter().map(Certificate).collect())
}
