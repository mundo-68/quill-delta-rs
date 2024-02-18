// Copyright 2024 quill-delta-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use crate::types::attr_val::AttrVal;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct AttrMap {
    #[serde(flatten)]
    map: HashMap<String, AttrVal>,
}

impl AttrMap {
    pub fn insert<K: Into<String>, V: Into<AttrVal>>(&mut self, key: K, val: V) {
        let k: String = key.into();
        let v: AttrVal = val.into();
        self.map.insert(k, v);
    }
}

impl Deref for AttrMap {
    type Target = HashMap<String, AttrVal>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
impl DerefMut for AttrMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

#[cfg(test)]
mod test {
    use crate::types::attr_map::AttrMap;
    use crate::types::attr_val::AttrVal;

    #[test]
    fn to_map_to_string_passes() {
        let mut map = AttrMap::default();
        map.insert("1".to_string(), AttrVal::Number(1));
        map.insert("2".to_string(), AttrVal::Bool(true));
        map.insert("3".to_string(), AttrVal::String("3".to_string()));

        let map2 = map.clone();
        map.insert("4".to_string(), AttrVal::Map(map2));

        let s = serde_json::to_string(&map).unwrap();
        dbg!(&s);
        let map3: AttrMap = serde_json::from_str(&s).unwrap();
        dbg!(map3);
    }
}
