pub use std::time::Duration;
pub use time::OffsetDateTime;

pub mod legal;
pub mod login;
pub mod meters;
pub mod status;
pub mod system_status;

mod serde;

type Float = f64;
