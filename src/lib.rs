//! Core library for the `robinrust` crate.
//!
//! This crate provides simple, async Rust bindings for a subset of Robinhood's
//! crypto endpoints, including authentication, account info, market data, and
//! trading utilities. See individual modules for details.

extern crate core;

mod auth;
mod account;
mod market_data;
mod trading;