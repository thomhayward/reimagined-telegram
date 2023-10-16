use serde::{Deserialize, Serialize};

/// Deserialized payload returned from the `/api/legal/radio` endpoint.
#[derive(Debug, Deserialize, Serialize)]
pub struct Radio {
	pub manufacturer: String,
	pub model: String,
	pub fcc_id: String,
	pub ic_id: String,
}
