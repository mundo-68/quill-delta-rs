// Copyright 2024 quill-delta-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Not a document. Documents only contain Insert-operations.")]
    NotADocument,
    #[error("Programming error: Trying to get the value of an attribute (type = {tpe:?}), but the wrong type is used.")]
    GetValueWrongType { tpe: String },
    #[error("Deserialization error: Detected nested Map-type (value = {value:?})")]
    SerdeNestedMap { value: String },
    #[error("Deserialization error: Detected unkown type (type = {tpe:?})")]
    SerdeUnknownType { tpe: String },
    #[error("Expected an attribute key (key {attr_key:?}), but could not find it.")]
    NotFoundAttributeKey { attr_key: String },
    #[error("Expected an unsigned integer")]
    NotAnUnsigned,
    #[error("Empty vector found when calculating min()")]
    EmptyVectorMinOp,
    #[error("Empty vector found when calculating last()")]
    EmptyVectorLastOp,
    #[error("Iterator has no next element")]
    IteratorIsEmpty,
}
