//! # Version Info
//!
//! Copyright (c) 2018, Cambridge Consultants Ltd.
//! See the top-level README.md for licence details.
//!
//! This module gives access to the auto-generated version string.

mod data;

pub fn version() -> &'static str {
    unsafe { ::core::str::from_utf8_unchecked(&data::VERSION_TEXT).trim() }
}
