#![warn(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations
)]
//! Collection of tools to interact with zentime server, clients, configuration and to handle
//! inter-process-communication.

pub mod client;
pub mod config;
pub mod ipc;
pub mod server;
