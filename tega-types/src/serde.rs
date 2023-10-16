pub mod duration {
	//! Module for serializing and deserializing [`Duration`] from strings of
	//! the form '{hours}h{minutes}m{seconds}s'.
	//!
	//! # Examples
	//! ```
	//! #[derive(serde::Deserialize, serde::Serialize)]
	//! struct Status {
	//!     #[serde(with = "tega_types::duration")]
	//!     uptime: std::time::Duration
	//! }
	//! ```
	//!
	//! [`Duration`]: std::time::Duration
	use std::{sync::OnceLock, time::Duration};

	const SECONDS_PER_HOUR: f64 = 3600f64;
	const SECONDS_PER_MINUTE: f64 = 60f64;

	static RE: OnceLock<regex::Regex> = OnceLock::new();

	pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		use serde::{de::Error, Deserialize};

		let re = RE.get_or_init(|| unsafe {
			// SAFETY: This regular expression is valid.
			//
			regex::Regex::new(r#"^(\d+)h(\d+)m([\d|\.]+)s$"#).unwrap_unchecked()
		});

		let s = String::deserialize(deserializer)?;
		let captures = re
			.captures(&s)
			.ok_or(Error::custom("unrecognised format for duration"))?;

		let (_, [hours, minutes, seconds]) = captures.extract();
		let hours: f64 = hours.parse().map_err(Error::custom)?;
		let minutes: f64 = minutes.parse().map_err(Error::custom)?;
		let seconds: f64 = seconds.parse().map_err(Error::custom)?;

		Ok(Duration::from_secs_f64(
			hours * SECONDS_PER_HOUR + minutes * SECONDS_PER_MINUTE + seconds,
		))
	}

	pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let total = duration.as_secs_f64();
		let (hours, remaining) = (
			total.div_euclid(SECONDS_PER_HOUR),
			total.rem_euclid(SECONDS_PER_HOUR),
		);
		let (minutes, seconds) = (
			remaining.div_euclid(SECONDS_PER_MINUTE),
			remaining.rem_euclid(SECONDS_PER_MINUTE),
		);

		serializer.serialize_str(&format!("{hours}h{minutes}m{seconds:.9}s"))
	}

	#[cfg(test)]
	mod tests {
		use super::*;
		use serde::{Deserialize, Serialize};

		#[derive(Debug, Deserialize, Serialize)]
		struct Test {
			#[serde(with = "crate::serde::duration")]
			value: Duration,
		}

		#[test]
		fn can_deserialize() {
			let s: Test = serde_json::from_str(r#"{"value":"12h54m37.123432s"}"#).unwrap();
			assert_eq!(s.value, Duration::from_secs_f64(46477.123432));
		}
	}
}

pub mod hash {
	//! Module for serializing and deserializing hex-encoded SHA1 hashes.
	//!
	//! # Examples
	//! ```
	//! #[derive(serde::Deserialize, serde::Serialize)]
	//! struct Status {
	//!    #[serde(with = "tega_types::hash")]
	//!    git_hash: [u8; 20]
	//! }
	//! ```
	use data_encoding::HEXLOWER;

	pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 20], D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		use serde::{de::Error, Deserialize};

		let hex_string = String::deserialize(deserializer)?;
		let mut dst = [0; 20];
		HEXLOWER
			.decode_mut(hex_string.as_bytes(), &mut dst)
			.map_err(|decode_error| Error::custom(format!("{decode_error:?}")))?;

		Ok(dst)
	}

	pub fn serialize<S>(hash: &[u8; 20], serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use std::str;

		let mut encoded = vec![0; 40];
		HEXLOWER.encode_mut(hash, &mut encoded);

		serializer.serialize_str(unsafe { str::from_utf8_unchecked(&encoded) })
	}
}
