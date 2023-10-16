use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// `/api/login/Basic`
#[derive(Debug, Deserialize, Serialize)]
pub struct LoginBasic {
	pub email: String,
	pub firstname: String,
	pub lastname: String,
	pub roles: Vec<String>,
	pub token: String,
	pub provider: String,
	#[serde(rename = "loginTime")]
	pub login_time: OffsetDateTime,
}
