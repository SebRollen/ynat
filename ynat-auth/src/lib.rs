// Common types shared between client and server
pub mod common;

// Client library (public API for ynat)
mod client;
mod error;

pub use client::{authenticate, DeviceIdStore, ServerAuthClient, Settings, TokenStore};
pub use common::{StoredToken, TokenPair};
pub use error::AuthError;

// Server modules (public for binary, internal for library)
#[cfg(feature = "server")]
pub mod server;
