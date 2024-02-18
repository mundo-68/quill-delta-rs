#[cfg(test)]
mod tests {
    //Test Strategy
    // In order to test we must first create a structure...but the generated string is different
    //from the starting string because of spaces, and map entry sequence.
    //So we cycle 1) JSON --> 2) Delta --> 3) JSON --> 4_ Delta ...
    //Then we compare 2) with 4) to see if we were able to get back valid Delta from the generated string.

    use delta::attributes::Attributes;
    use delta::delta::Delta;
    use delta::operations::{DeltaOperation, OpsMap, OpsVal};
    use delta::types::ops_kind::OpKind;

    #[test]
    fn attr_to_string_passes() {
        let json = r#"{ "color": "red" , "imagine" :"dragons" }"#;
        let a1: Attributes = serde_json::from_str(json).unwrap();
        dbg!(&a1);
        assert_eq!(a1.keys().len(), 2);
        assert_eq!(a1.get("color").unwrap().str_val().unwrap(), "red");
        assert_eq!(a1.get("imagine").unwrap().str_val().unwrap(), "dragons");

        let s1 = serde_json::to_string(&a1).unwrap();
        let a2: Attributes = serde_json::from_str(&s1).unwrap();
        assert_eq!(a2.len(), 2);
        assert_eq!(a2.get("color").unwrap().str_val().unwrap(), "red");
        assert_eq!(a2.get("imagine").unwrap().str_val().unwrap(), "dragons");

        let json_data = r#"{ "font": { "family": "Helvetica", "size": "15px" }}"#;
        let a3: Attributes = serde_json::from_str(json_data).unwrap();
        dbg!(&a3);
        assert_eq!(a3.len(), 1);
        assert_eq!(
            a3.get("font")
                .unwrap()
                .map_val()
                .unwrap()
                .get("family")
                .unwrap()
                .str_val()
                .unwrap(),
            "Helvetica"
        );
        let s2 = serde_json::to_string(&a3).unwrap();
        dbg!(&s2);
        let a4: Attributes = serde_json::from_str(&s2).unwrap();
        assert_eq!(a4.len(), 1);
        assert_eq!(
            a4.get("font")
                .unwrap()
                .map_val()
                .unwrap()
                .get("family")
                .unwrap()
                .str_val()
                .unwrap(),
            "Helvetica"
        );
    }

    #[test]
    fn ops_to_string_passes() {
        let json = r#"{ "insert" : "hello world", "attributes": {} }"#;
        let opj1: DeltaOperation = serde_json::from_str(json).unwrap();
        assert!(opj1.get_attributes().is_empty());
        assert_eq!(opj1.op_len(), 11);
        assert!(opj1.insert_value().is_string());

        let json1 = serde_json::to_string(&opj1).unwrap();
        let opj2: DeltaOperation = serde_json::from_str(&json1).unwrap();
        assert!(opj2.get_attributes().is_empty());
        assert_eq!(opj2.op_len(), 11);
        assert_eq!(opj2.insert_value().is_string(), true);
        assert_eq!(opj2.insert_value(), opj1.insert_value());

        let json3 = r#"{ "insert" : "hello world", "attributes": { "color": "red" , "imagine" :"dragons" } }"#;
        let opj5: DeltaOperation = serde_json::from_str(json3).unwrap();
        assert_eq!(opj5.get_attributes().len(), 2);
        assert_eq!(opj5.op_len(), 11);
        assert!(opj5.insert_value().is_string());

        let json4 = serde_json::to_string(&opj5).unwrap();
        let opj7: DeltaOperation = serde_json::from_str(&json4).unwrap();
        assert_eq!(opj7.op_len(), 11); //hello world
        assert_eq!(opj7.insert_value().str_val().unwrap(), "hello world");
        assert_eq!(opj7.get_attributes().len(), 2);
        let c: &Attributes = opj7.get_attributes();
        assert_eq!(c.get("color").unwrap().str_val().unwrap(), "red");
        assert_eq!(c.get("imagine").unwrap().str_val().unwrap(), "dragons");

        let mut opj9 = Delta::default();
        opj9.insert(OpsVal::from(5));
        let json5 = serde_json::to_string(&opj9).unwrap();
        let opj10: Delta = serde_json::from_str(&json5).unwrap();
        assert_eq!(
            opj10.get(0).unwrap().get_op_kind(),
            &OpKind::Insert(OpsVal::Number(5))
        );

        let mut o = OpsMap::default();
        o.insert("imagine".to_string(), "dragons");
        let mut opj11 = Delta::default();
        opj11.insert(OpsVal::from(o));
        let json6 = serde_json::to_string(&opj11).unwrap();
        let opj12: Delta = serde_json::from_str(&json6).unwrap();
        dbg!(&opj12);
        let mut o: OpsMap = OpsMap::default();
        o.insert("imagine".to_string(), "dragons");
        assert_eq!(
            opj12.get(0).unwrap().get_op_kind(),
            &OpKind::Insert(OpsVal::Map(o))
        );
    }

    #[test]
    fn delta_from_string_passes() {
        let json = r##"
        {"ops" : [
            {"insert": "I am just a single sentence with "},
            {"attributes":{"bold":true}, "insert":"bolded, "},
            {"attributes":{"italic":true}, "insert":"italicized"},
            {"insert":", and "},
            {"insert":".\n"}
            ]}
        "##;
        dbg!(&json);
        let delta: Delta = serde_json::from_str(&json).unwrap();
        dbg!(&delta);
        assert_eq!(delta.len(), 5);

        let json2 = serde_json::to_string(&delta).unwrap();
        let delta2: Delta = serde_json::from_str(&json2).unwrap();

        let op1 = delta2.get(1).unwrap();
        assert_eq!(
            op1.get_op_kind(),
            &OpKind::Insert(OpsVal::String("bolded, ".to_owned()))
        );
        assert!(op1
            .get_attributes()
            .get("bold")
            .unwrap()
            .bool_val()
            .unwrap());
    }
}
