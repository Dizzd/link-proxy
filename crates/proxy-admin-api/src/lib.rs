macro_rules! ensure { ($condition:expr, $($message:tt)*) => { if !$condition { return Err(anyhow::anyhow!($($message)*).into()); } }; }

mod auth;
mod handlers;
pub mod router;
pub use router::{router, validate_configuration};
mod dto;
mod error;
mod state;
pub use handlers::auth::bootstrap_admin;
