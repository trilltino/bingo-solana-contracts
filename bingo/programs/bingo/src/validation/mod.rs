//! # Validation Module
//!
//! Centralized guards and validators that protect every instruction path.
//! Splitting the logic into focused submodules keeps instruction handlers
//! lightweight and encourages reuse.

pub mod access;
pub mod amount;
pub mod arithmetic;
pub mod guards;
pub mod input;

pub use access::*;
pub use amount::*;
pub use arithmetic::*;
pub use guards::*;
pub use input::*;
