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

pub mod initialize;
pub mod update_global_config;
pub mod initialize_token_registry;
pub mod add_approved_token;
pub mod remove_approved_token;
pub mod recover_room;
pub mod set_emergency_pause;

// Account structs are in lib.rs for Anchor macro compatibility
