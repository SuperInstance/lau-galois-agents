//! # lau-galois-agents
//!
//! Galois theory applied to agent capability spaces.
//!
//! The core insight: an agent's capability space is a "field", extending it with new capabilities
//! is a "field extension", and the symmetries preserving behavior form the "Galois group".
//! The fundamental theorem of Galois theory then tells us which capability configurations
//! are constructible and which are fundamentally impossible.

pub mod poset;
pub mod galois_connection;
pub mod field;
pub mod extension;
pub mod galois_group;
pub mod fundamental_theorem;
pub mod normal;
pub mod solvable;
pub mod splitting;
pub mod fixed;
pub mod constructible;
pub mod agent;

pub use poset::*;
pub use galois_connection::*;
pub use field::*;
pub use extension::*;
pub use galois_group::*;
pub use fundamental_theorem::*;
pub use normal::*;
pub use solvable::*;
pub use splitting::*;
pub use fixed::*;
pub use constructible::*;
pub use agent::*;
