#![allow(
    dead_code,
    reason = "Models should match database schema if fields are not used yet"
)]

pub mod admin;
pub mod core;
pub mod quiz;

// Re-export all models for easy access
pub use admin::*;
pub use core::*;
pub use quiz::*;
