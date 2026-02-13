pub mod activations;
pub mod builder;
pub mod config;

// Re-export serde helpers for macro-generated code
// This allows the hub_methods macro to reference serde helpers via crate::serde_helpers
pub use plexus_core::serde_helpers;

// Re-export plexus module for macro-generated code
pub use plexus_core::plexus;

// Re-export the hub builder functions
pub use builder::{build_comms_hub, build_default_hub, build_from_config_file};
pub use config::CommsConfig;

// Re-export commonly used types for convenience
pub use plexus_core::plexus::{Activation, DynamicHub, PlexusError, PlexusStream};
