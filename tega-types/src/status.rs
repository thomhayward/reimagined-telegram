use serde::{Deserialize, Serialize};
use std::time::Duration;
use time::{format_description::FormatItem, OffsetDateTime};

/// `/api/status`
#[derive(Debug, Deserialize, Serialize)]
pub struct Status {
	pub din: String,
	#[serde(with = "datetime")]
	pub start_time: OffsetDateTime,
	#[serde(rename = "up_time_seconds", with = "crate::serde::duration")]
	pub up_time: Duration,
	pub is_new: bool,
	pub version: String,
	#[serde(with = "crate::serde::hash")]
	pub git_hash: [u8; 20],
	pub commission_count: u16,
	pub device_type: String,
	pub teg_type: String,
	pub sync_type: String,
	pub leader: String,
	pub followers: Option<String>,
	pub cellular_disabled: bool,
}

pub const DATETIME_FORMAT: &[FormatItem<'_>] = time::macros::format_description!(
	"[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]"
);

time::serde::format_description!(datetime, OffsetDateTime, DATETIME_FORMAT);

#[cfg(test)]
mod tests {
	use super::Status;

	#[test]
	fn deserialize_status() {
		let sample = include_bytes!("../samples/api-status.json");
		let status: Status = serde_json::from_slice(sample).unwrap();

		assert_eq!(status.din, "1152100-13-J--AB123456C7D8EF");
		assert_eq!(status.version, "23.12.11 452c76cb");
	}
}
