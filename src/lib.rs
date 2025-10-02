//! Core library for the `robinrust` crate.
//!
//! This crate provides simple, async Rust bindings for a subset of Robinhood's
//! crypto endpoints, including authentication, account info, market data, and
//! trading utilities. See individual modules for details.

extern crate core;

pub mod auth;
pub mod account;
pub mod market_data;
pub mod trading;