// Copyright 2024 quill-delta-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::operations::OpsVal;
use serde_derive::{Deserialize, Serialize};
#[cfg(test)]
use std::fmt;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpKind {
    #[serde(rename = "insert")]
    Insert(OpsVal),
    #[serde(rename = "retain")]
    Retain(usize),
    #[serde(rename = "delete")]
    Delete(usize),
}

impl From<String> for OpKind {
    fn from(s: String) -> Self {
        OpKind::Insert(OpsVal::String(s))
    }
}

impl From<&str> for OpKind {
    fn from(s: &str) -> Self {
        OpKind::Insert(OpsVal::String(s.to_owned()))
    }
}

impl From<usize> for OpKind {
    fn from(s: usize) -> Self {
        OpKind::Insert(OpsVal::Number(s))
    }
}

// impl From<HashMap<String,Attributes>> for OpKind {
//     fn from(s:HashMap<String,Attributes>) -> Self {
//         let m = OpsMap::new();
//         OpKind::Insert(OpsVal::Map(m))
//     }
// }

#[cfg(test)]
impl fmt::Display for OpKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpKind::Insert(u) => {
                write!(f, "Insert({u})")
            }
            OpKind::Retain(s) => {
                write!(f, "Retain({s})")
            }
            OpKind::Delete(b) => {
                write!(f, "Delete({b})")
            }
        }
    }
}
