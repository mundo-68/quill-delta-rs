// Copyright 2024 quill-delta-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! # Delta
//!
//! Deltas are a simple, yet expressive document format that can be used to describe contents and changes.
//! The format is JSON based, and is human readable, yet easily parsable by machines.
//! Deltas can describe any rich text document, includes all text and formatting information,
//! without the ambiguity and complexity of HTML.
//!
//! A Delta is made up of an Array of Operations, which describe changes to a document.
//! They can be an insert, delete or retain. Note operations do not take an index.
//! They always describe the change at the current index. Use retains to "keep" or "skip" certain
//! parts of the document.
//!
//! Don’t be confused by its name Delta—Deltas represents both documents and changes to documents.
//! If you think of Deltas as the instructions from going from one document to another,
//! the way Deltas represent a document is by expressing the instructions starting from
//! an empty document.

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
