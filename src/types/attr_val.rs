// Copyright 2024 quill-delta-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.


use crate::error::Error;
use crate::error::Error::{GetValueWrongType, SerdeNestedMap, SerdeUnknownType};
use crate::types::attr_map::AttrMap;
use anyhow::Result;
use serde_derive::Serialize;
use serde_json::Value;
#[cfg(test)]
use std::fmt;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(try_from = "Value")]
#[serde(untagged)]
pub enum AttrVal {
    String(String),
    Number(usize),
    Bool(bool),
    Map(AttrMap),
    Null,
}

impl AttrVal {
    /// # Errors
    /// `GetValueWrongType` when the `AttrVal` does not contain this type
    pub fn str_val(&self) -> anyhow::Result<&str, Error> {
        if let AttrVal::String(s) = self {
            return Ok(s.as_str());
        }
        Err(GetValueWrongType {
            tpe: "string".to_string(),
        })
    }
    /// # Errors
    /// `GetValueWrongType` when the `AttrVal` does not contain this type
    pub fn number_val(&self) -> Result<usize, Error> {
        if let AttrVal::Number(s) = self {
            return Ok(*s);
        }
        Err(GetValueWrongType {
            tpe: "number".to_string(),
        })
    }

    /// Note: Calling this function on a String(), or Number() will result in None too
    /// # Errors
    /// `GetValueWrongType` when the `AttrVal` does not contain this type
    pub fn map_val(&self) -> Result<&AttrMap, Error> {
        if let AttrVal::Map(s) = self {
            return Ok(s);
        }
        Err(GetValueWrongType {
            tpe: "map".to_string(),
        })
    }

    /// # Errors
    /// `GetValueWrongType` when the `AttrVal` does not contain this type
    pub fn bool_val(&self) -> Result<bool, Error> {
        if let AttrVal::Bool(s) = self {
            return Ok(*s);
        }
        Err(GetValueWrongType {
            tpe: "boolean".to_string(),
        })
    }

    pub fn is_string(&self) -> bool {
        if let AttrVal::String(_) = self {
            return true;
        }
        false
    }
    pub fn is_number(&self) -> bool {
        if let AttrVal::Number(_) = self {
            return true;
        }
        false
    }
    pub fn is_bool(&self) -> bool {
        if let AttrVal::Bool(_) = self {
            return true;
        }
        false
    }
    pub fn is_null(&self) -> bool {
        if let AttrVal::Null = self {
            return true;
        }
        false
    }
    pub fn is_map(&self) -> bool {
        if let AttrVal::Map(_) = self {
            return true;
        }
        false
    }
}

impl From<String> for AttrVal {
    fn from(s: String) -> Self {
        AttrVal::String(s)
    }
}

impl From<usize> for AttrVal {
    fn from(s: usize) -> Self {
        AttrVal::Number(s)
    }
}

impl From<bool> for AttrVal {
    fn from(s: bool) -> Self {
        AttrVal::Bool(s)
    }
}

impl From<&str> for AttrVal {
    fn from(s: &str) -> Self {
        AttrVal::String(s.to_string())
    }
}

impl From<AttrMap> for AttrVal {
    fn from(s: AttrMap) -> Self {
        AttrVal::Map(s)
    }
}

impl TryFrom<Value> for AttrVal {
    type Error = Error;
    fn try_from(s: Value) -> Result<Self, Self::Error> {
        serde_val_to_attr_val(s, true)
    }
}

#[allow(clippy::cast_possible_truncation)]
fn serde_val_to_attr_val(value: Value, allow_nesting: bool) -> Result<AttrVal, Error> {
    match value {
        Value::Null => Ok(AttrVal::Null),
        Value::String(s) => Ok(AttrVal::String(s)),
        Value::Bool(b) => Ok(AttrVal::Bool(b)),
        Value::Number(n) => {
            let Some(nn) = n.as_u64() else {
                return Err(Error::NotAnUnsigned);
            };
            Ok(AttrVal::Number(nn as usize))
        }
        Value::Object(o) => {
            if allow_nesting {
                Ok(AttrVal::Map(serde_val_to_map(o, allow_nesting)?))
            } else {
                Err(SerdeNestedMap {
                    value: Value::Object(o).to_string(),
                })
            }
        }
        Value::Array(_) => Err(SerdeUnknownType {
            tpe: value.to_string(),
        }),
    }
}

#[allow(clippy::cast_possible_truncation)]
fn serde_val_to_map(
    value: serde_json::map::Map<String, Value>,
    allow_nesting: bool,
) -> Result<AttrMap, Error> {
    let mut att = AttrMap::default();
    for (kk, vv) in value {
        let k = kk.to_string();
        let v = match vv {
            Value::Null => AttrVal::Null,
            Value::String(s) => AttrVal::String(s),
            Value::Bool(b) => AttrVal::Bool(b),
            Value::Number(n) => {
                let Some(nn) = n.as_u64() else {
                    return Err(Error::NotAnUnsigned);
                };
                AttrVal::Number(nn as usize)
            }
            Value::Object(o) => {
                if allow_nesting {
                    serde_val_to_attr_val(Value::Object(o), allow_nesting)?
                } else {
                    return Err(SerdeNestedMap {
                        value: Value::Object(o).to_string(),
                    });
                }
            }
            Value::Array(_) => {
                return Err(SerdeUnknownType {
                    tpe: vv.to_string(),
                })
            }
        };
        att.insert(k, v);
    }

    Ok(att)
}

#[cfg(test)]
impl fmt::Display for AttrVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_attrval(self, f)
    }
}
#[cfg(test)]
fn fmt_attrval(attrval: &AttrVal, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match attrval {
        AttrVal::Null => {
            write!(f, "Null")
        }
        AttrVal::Number(u) => {
            write!(f, "{u}")
        }
        AttrVal::String(s) => {
            write!(f, "{s}")
        }
        AttrVal::Bool(b) => {
            write!(f, "{b}")
        }
        AttrVal::Map(m) => {
            let mut out = String::new();
            for (k, v) in &**m {
                out.push_str(&format!("({k}->{v}), "));
            }
            write!(f, "{out}")
        }
    }
}

#[cfg(test)]
mod test {
    use crate::attributes::Attributes;
    use crate::types::attr_val::{AttrMap, AttrVal};
    use log::warn;

    #[test]
    fn attr_val_from_x_passes() {
        let val = AttrVal::from("I am a test");
        warn!("Unsupported format: {}", val);

        let val = AttrVal::from(true);
        warn!("Unsupported format: {}", val);

        let val = AttrVal::Null;
        warn!("Unsupported format: {}", val);

        let val = AttrVal::from(42);
        warn!("Unsupported format: {}", val);

        let mut m = AttrMap::default();
        m.insert("number".to_string(), 42);
        m.insert("string".to_string(), "forty two");
        m.insert("null".to_string(), AttrVal::Null);
        let val = AttrVal::from(m);
        warn!("Unsupported format: {}", &val);
    }

    #[test]
    fn attr_val_with_map_to_sting_passes() {
        let mut attrib = Attributes::default();

        let val = AttrVal::from("I am a test");
        attrib.insert("1", val);
        let val = AttrVal::from(true);
        attrib.insert("2", val);
        let val = AttrVal::Null;
        attrib.insert("3", val);
        let val = AttrVal::from(42);
        attrib.insert("4", val);

        let mut map = AttrMap::default();
        map.insert("1".to_string(), AttrVal::Number(1));
        map.insert("2".to_string(), AttrVal::Bool(true));
        map.insert("3".to_string(), AttrVal::String("3".to_string()));

        let map2 = map.clone();
        map.insert("4".to_string(), AttrVal::Map(map2));
        attrib.insert("5", map);

        let s = serde_json::to_string(&attrib).unwrap();
        assert!(!s.contains("attr"));
        assert!(!s.contains("attr"));
        let _map3: AttrMap = serde_json::from_str(&s).unwrap();
    }
}
