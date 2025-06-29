#![allow(unused_imports)]

use pax_kit::*;

pub mod streaming_server;
pub mod reactive_client;

pub use reactive_client::ReactiveStreamingComponent;

// Re-export for external use
pub use streaming_server::{ReactiveStreamingServer, ComponentUpdate, UpdateType};
pub use reactive_client::{ReactiveClient, UpdateHandler, ExternalUpdateSource};