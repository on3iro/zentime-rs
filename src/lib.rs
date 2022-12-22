#![warn(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations
)]
//! Zentime is a client/server based CLI pomodor/productivity timer written in Rust.
//! This crate consists of a binary and a library crate.
//! The library crate is a collection of tools to interact with zentime server, clients, configuration and to handle
//! inter-process-communication.
//!
#![doc = include_str!("../README.md")]

pub mod client;
pub mod config;
pub mod ipc;
pub mod server;
