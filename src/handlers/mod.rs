pub mod admin;
pub mod quiz;
pub mod web;

// Re-export all handlers for easy access
pub use admin::*;
pub use quiz::*;
pub use web::*;