use chrono::{DateTime, Utc};
use uuid::Uuid;
pub mod action;
pub mod condition;
pub mod domain;
pub mod error;
pub mod mapping;
pub mod matcher;
pub mod revision;
pub mod rule;
pub mod upstream;
pub use action::*;
pub use condition::*;
pub use domain::*;
pub use mapping::*;
pub use matcher::*;
pub use revision::*;
pub use rule::*;
pub use upstream::*;
pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}
pub fn yes() -> bool {
    true
}
pub fn now() -> DateTime<Utc> {
    Utc::now()
}
