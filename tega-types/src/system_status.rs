use crate::Float;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Payload returned from the `/api/system_status` endpoint.
#[derive(Debug, Deserialize, Serialize)]
pub struct SystemStatus {
	pub command_source: String,

	/// Target power in Watts.
	///
	/// Negative values indicate the battery is charging.
	pub battery_target_power: Float,
	pub battery_target_reactive_power: Float,
	pub nominal_full_pack_energy: u32,
	pub nominal_energy_remaining: u32,

	pub max_power_energy_remaining: u32,
	pub max_power_energy_to_be_charged: u32,
	pub max_charge_power: u32,
	pub max_discharge_power: u32,
	pub max_apparent_power: u32,
	pub instantaneous_max_discharge_power: u32,
	pub instantaneous_max_charge_power: u32,
	//
	pub grid_services_power: Float,
	pub system_island_state: String,
	pub available_blocks: u32,
	pub battery_blocks: Vec<BatteryBlock>,

	pub ffr_power_availability_high: u32,
	pub ffr_power_availability_low: u32,
	pub load_charge_constraint: u32,
	pub max_sustained_ramp_rate: u32,

	pub grid_faults: Vec<String>,
	pub can_reboot: String,

	pub smart_inv_delta_p: u32,
	pub smart_inv_delta_q: u32,

	#[serde(with = "time::serde::rfc3339")]
	pub last_toggle_timestamp: OffsetDateTime,

	pub solar_real_power_limit: i32,
	pub score: i32,

	pub blocks_controlled: u16,
	pub primary: bool,
	pub auxiliary_load: u32,

	pub all_enable_lines_high: bool,
	pub inverter_nominal_usable_power: u32,
	pub expected_energy_remaining: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BatteryBlock {
	#[serde(rename = "Type")]
	pub ty: String,

	#[serde(rename = "PackagePartNumber")]
	pub package_part_number: String,

	#[serde(rename = "PackageSerialNumber")]
	pub package_serial_number: String,

	pub disabled_reasons: Vec<String>,

	pub pinv_state: String,
	pub pinv_grid_state: String,

	pub nominal_energy_remaining: u32,
	pub nominal_full_pack_energy: u32,

	pub p_out: i32,
	pub q_out: i32,
	pub v_out: Float,
	pub f_out: Float,
	pub i_out: Float,

	pub energy_charged: u32,
	pub energy_discharged: u32,

	pub off_grid: bool,
	pub vf_mode: bool,
	pub wobble_detected: bool,
	pub charge_power_clamped: bool,
	pub backup_ready: bool,

	#[serde(rename = "OpSeqState")]
	pub op_seq_state: String,
	pub version: String,
}

/// Payload returned from the `/api/system_status/soe` endpoint.
///
/// `percentage` is the aggregated charged state in percent of all the
/// Powerwalls in the system.
#[derive(Debug, Deserialize, Serialize)]
pub struct StateOfEnergy {
	pub percentage: f64,
}

#[cfg(test)]
mod tests {
	use super::SystemStatus;

	#[test]
	fn deserialize_system_status() {
		let sample = include_bytes!("../samples/api-system_status.json");
		let _: SystemStatus = serde_json::from_slice(sample).unwrap();
	}
}
