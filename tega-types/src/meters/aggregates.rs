use std::iter;

use crate::{Float, OffsetDateTime};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AggregateClass {
	Load,
	Grid,
	Battery,
	Solar,
}

/// Payload returned from the `/api/meters/aggregates` endpoint.
#[derive(Debug, Deserialize, Serialize)]
pub struct MetersAggregates {
	/// "Home" in the Tesla mobile app.
	///
	/// Positive numbers indicate power draw from the system to the home.
	/// Negative numbers should never happen.
	pub load: AggregateMeterDevice,

	/// "Grid" in the Tesla mobile app.
	///
	/// Positive numbers indicate power draw from the grid to the system.
	/// Negative numbers indicate sending power from the system to the grid
	#[serde(rename = "site")]
	pub grid: AggregateMeterDevice,

	/// "Powerwall" in the Tesla mobile app. This is an aggregate number if you
	/// have more than one Powerwall.
	///
	/// Positive numbers indicate power draw from the batteries to the system.
	/// Negative numbers indicate sending power from the system to the batteries
	pub battery: AggregateMeterDevice,

	/// "Solar" in the Tesla mobile app.
	///
	/// Positive numbers indicate power production from solar to the system.
	/// Negative numbers indicate sending power from the system to solar.
	pub solar: AggregateMeterDevice,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AggregateMeterDevice {
	#[serde(with = "time::serde::rfc3339")]
	pub last_communication_time: OffsetDateTime,

	pub timeout: u64,

	/// Total power in Watts.
	///
	/// A positive value indicates power draw from the device; a negative value
	/// indicates power flow to the device.
	pub instant_power: Float,
	pub instant_reactive_power: Float,
	pub instant_apparent_power: Float,

	/// Average voltage in Volts.
	pub instant_average_voltage: Float,

	/// Total current in Amps.
	pub instant_total_current: Float,

	/// AC voltage frequency in Hz.
	pub frequency: Float,

	pub i_a_current: Float,
	pub i_b_current: Float,
	pub i_c_current: Float,

	pub energy_exported: Float,
	pub energy_imported: Float,

	#[serde(with = "time::serde::rfc3339")]
	pub last_phase_voltage_communication_time: OffsetDateTime,

	#[serde(with = "time::serde::rfc3339")]
	pub last_phase_power_communication_time: OffsetDateTime,

	#[serde(with = "time::serde::rfc3339")]
	pub last_phase_energy_communication_time: OffsetDateTime,

	/// The number of meters this aggregation covers.
	///
	/// If this isn't present in the raw response from the API, the number of
	/// meters is assumed to be `1`.
	#[serde(default = "default_num_meters")]
	pub num_meters_aggregated: u16,
}

impl MetersAggregates {
	pub fn sinks(&self) -> impl Iterator<Item = (AggregateClass, &AggregateMeterDevice)> {
		assert!(self.load.instant_power.is_sign_positive());

		let sinks = [
			// The load is always a sink.
			Some((AggregateClass::Load, &self.load)),
			self.grid
				.instant_power
				.is_sign_negative()
				.then_some((AggregateClass::Grid, &self.grid)),
			self.battery
				.instant_power
				.is_sign_negative()
				.then_some((AggregateClass::Battery, &self.battery)),
			self.solar
				.instant_power
				.is_sign_negative()
				.then_some((AggregateClass::Solar, &self.solar)),
		];

		let mut index = 0..sinks.len();

		iter::from_fn(move || {
			while let Some(index) = index.next() {
				if let Some(sink) = sinks[index] {
					return Some(sink);
				}
			}
			None
		})
	}

	pub fn sources(&self) -> impl Iterator<Item = (AggregateClass, &AggregateMeterDevice)> {
		let sinks = [
			self.grid
				.instant_power
				.is_sign_positive()
				.then_some((AggregateClass::Grid, &self.grid)),
			self.battery
				.instant_power
				.is_sign_positive()
				.then_some((AggregateClass::Battery, &self.battery)),
			self.solar
				.instant_power
				.is_sign_positive()
				.then_some((AggregateClass::Solar, &self.solar)),
		];

		let mut index = 0..sinks.len();

		iter::from_fn(move || {
			while let Some(index) = index.next() {
				if let Some(sink) = sinks[index] {
					return Some(sink);
				}
			}

			None
		})
	}
}

fn default_num_meters() -> u16 {
	1
}

#[cfg(test)]
mod tests {
	use super::MetersAggregates;

	#[test]
	fn parse_meters_aggregates_sample() {
		let sample = include_bytes!("../../samples/api-meters-aggregates.json");
		let meters: MetersAggregates = serde_json::from_slice(sample).unwrap();

		assert_eq!(meters.sources().count(), 2);
		assert_eq!(meters.sinks().count(), 2);

		let total_generation: f64 = meters
			.sources()
			.map(|(_, device)| device.instant_reactive_power)
			.sum();
		let total_usage: f64 = meters
			.sinks()
			.map(|(_, device)| device.instant_reactive_power.abs())
			.sum();

		dbg!(total_generation, total_usage);
	}
}
