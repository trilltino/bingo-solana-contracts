//! # Instructions Module
//!
//! All program instructions organized into dedicated feature-based modules.
//!
//! ## Module Organization
//!
//! Instructions are organized by feature area for better maintainability and scalability:
//!
//! - **admin**: Platform configuration (initialize, token registry, pause)
//! - **room**: Room creation and management (init_pool_room, init_asset_room)
//! - **player**: Player participation (join_room, ready_up, leave_room)
//! - **game**: Game execution (declare_winners, end_room)
//! - **utils**: Shared utility functions (BPS calculations, validation helpers)
//!
//! ## Design Philosophy
//!
//! This feature-based organization follows industry best practices:
//! - Clear separation of concerns (admin vs player vs game operations)
//! - Easy to locate relevant code (all game-ending logic in game/ module)
//! - Scalable for future features (recovery/, governance/, etc.)
//! - Mirrors frontend feature-based structure for consistency

pub mod admin;
pub mod room;
pub mod player;
pub mod game;
pub mod asset;
pub mod utils;

// Context structs are now in lib.rs for Anchor macro compatibility
