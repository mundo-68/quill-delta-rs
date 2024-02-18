// Copyright 2024 quill-delta-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]

#[cfg(feature = "json")]
extern crate serde;
#[cfg(feature = "json")]
#[macro_use]
extern crate serde_derive;

//Data types supporting the delta document format
pub mod types;

//Delta document format definition
pub mod attributes;
pub mod delta;
pub mod operations;

//Operations on the delta document
pub mod document;
mod error;
pub mod iterator;
pub mod optransform;
pub mod utils;
