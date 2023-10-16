use clap::Parser;
use rustls::Certificate;
use std::{
	fs::File,
	io::{self, BufReader},
	net::IpAddr,
	path::{Path, PathBuf},
};
use tega_client::Client;
// use time::OffsetDateTime;

#[derive(Parser)]
struct Arguments {
	ip_address: IpAddr,
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
