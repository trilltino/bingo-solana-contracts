//! # Admin Instructions Module
//!
//! Administrative instructions for platform configuration and management.
//!
//! This module contains privileged operations that can only be executed by the
//! platform admin. These instructions configure platform-wide settings that all
//! rooms must follow.
//!
//! ## Instructions
//!
//! - **initialize**: One-time setup of GlobalConfig (platform wallets, fee structure)
//!
//! ## Future Admin Instructions
//!
//! - **add_approved_token**: Add SPL token to allowlist
//! - **remove_approved_token**: Remove SPL token from allowlist
//! - **update_fees**: Modify platform fee structure (requires governance)
//! - **emergency_pause**: Circuit breaker for security incidents
//! - **update_admin**: Transfer admin authority

pub mod add_approved_token;
pub mod initialize;
pub mod initialize_token_registry;
pub mod recover_room;
pub mod remove_approved_token;
pub mod set_emergency_pause;
pub mod update_global_config;

// Re-export Account structs for use in lib.rs
pub use add_approved_token::AddApprovedToken;
pub use initialize::Initialize;
pub use initialize_token_registry::InitializeTokenRegistry;
pub use recover_room::RecoverRoom;
pub use remove_approved_token::RemoveApprovedToken;
pub use set_emergency_pause::SetEmergencyPause;
pub use update_global_config::UpdateGlobalConfig;
