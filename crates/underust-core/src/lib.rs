//! Core logic for the underust harness.
//!
//! This crate performs no process invocation and no printing. Everything it does is
//! pure logic over paths and strings, so its tests pass on a machine with no Rust
//! toolchain available to compile a task.

pub mod host;
pub mod manifest;
pub mod requires;
pub mod seal;
