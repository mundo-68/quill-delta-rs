#[cfg(test)]
mod test {
    use anyhow::Result;
    use delta::attributes::Attributes;
    use delta::delta::Delta;
    use delta::operations::{DeltaOperation, OpsVal};
    use delta::types::ops_kind::OpKind;

    #[test]
    fn to_attr_passes() -> Result<()> {
        let _: Attributes = serde_json::from_str(r#"{"attributes": { "color": "red" }}"#)?;
        let _: Attributes = serde_json::from_str(r#"{"attributes": { "italic": null }}"#)?;
        let _: Attributes = serde_json::from_str(r#"{"attributes": { "bold": true }}"#)?;

        let a: Attributes =
            serde_json::from_str(r#"{"attributes": { "color": "red" , "imagine" :"dragons" }}"#)?;
        //dbg!(&a);
        let map = a.get("attributes").unwrap().map_val().unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("color").unwrap().str_val()?, "red");
        assert_eq!(map.get("imagine").unwrap().str_val()?, "dragons");

        let json_data = r#"{"attributes": { "font": { "family": "Helvetica", "size": "15px" }}}"#;
        let a2: Attributes = serde_json::from_str(json_data)?;
        let map = a2.get("attributes").unwrap().map_val().unwrap();
        assert_eq!(map.len(), 1);
        assert_eq!(
            map.get("font")
                .unwrap()
                .map_val()
                .unwrap()
                .get("family")
                .unwrap()
                .str_val()?,
            "Helvetica"
        );

        Ok(())
    }

    #[test]
    fn to_ops_passes() {
        let json = r#"{ "insert" : "hello world", "attributes": {} }"#;
        let opj1: DeltaOperation = serde_json::from_str(json).unwrap();
        assert_eq!(opj1.get_attributes().len(), 0);
        assert_eq!(opj1.op_len(), 11); //hello world

        let json = r#"{ "insert" : "hello world", "attributes": { "color": "red" , "imagine" :"dragons" } }"#;
        let opj2: DeltaOperation = serde_json::from_str(json).unwrap();
        assert_eq!(opj2.get_attributes().len(), 2);
        assert_eq!(opj2.op_len(), 11);
        let c: &Attributes = opj2.get_attributes();
        assert_eq!(c.get("color").unwrap().str_val().unwrap(), "red");
        assert_eq!(c.get("imagine").unwrap().str_val().unwrap(), "dragons");
    }

    #[test]
    fn to_delta_passes() {
        //multiple operations
        let json = r#"{"ops" : [
            {"insert":"I am just a single sentence with "},
            {"attributes":{"bold":true},"insert":"bolded, "},
            {"attributes":{"italic":true},"insert":"italicized"},
            {"insert":", and "},
            {"insert":".\\n"}
            ]}"#;
        let delta: Delta = serde_json::from_str(json).unwrap();
        assert_eq!(delta.len(), 5);
        let op1 = delta.get(1).unwrap();
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

        //single operation
        let json2 =
            r#"{"ops" : [{"insert":"Hallo","attributes":{"font":"green","hello":true} }] }"#;
        let delta2: Delta = serde_json::from_str(json2).unwrap();
        assert_eq!(
            delta2.first().unwrap().get_op_kind(),
            &OpKind::Insert("Hallo".to_string().into())
        );
        assert!(delta2
            .first()
            .unwrap()
            .get_attributes()
            .get("hello")
            .unwrap()
            .bool_val()
            .unwrap(),);

        assert_eq!(
            delta2
                .first()
                .unwrap()
                .get_attributes()
                .get("font")
                .unwrap()
                .str_val()
                .unwrap(),
            "green"
        );

        //without attributes
        let json22 = r#"{"ops" : [{"insert":"Hallo"}] }"#;
        let delta22: Delta = serde_json::from_str(json22).unwrap();
        assert_eq!(
            delta22.first().unwrap().get_op_kind(),
            &OpKind::Insert("Hallo".to_string().into())
        );

        //other operations...
        let json3 = r#"{"ops" : [{"delete":10, "attributes":{"font":"green","hello":"world"} }] }"#;
        let delta3: Delta = serde_json::from_str(json3).unwrap();
        assert_eq!(delta3.first().unwrap().get_op_kind(), &OpKind::Delete(10));

        let json4 = r#"{"ops" : [{"retain":9,"attributes":{"font":"green","hello":"world"} }] }"#;
        let delta4: Delta = serde_json::from_str(json4).unwrap();
        assert_eq!(delta4.first().unwrap().get_op_kind(), &OpKind::Retain(9));
    }

    #[test]
    fn longer_json_text_deserialization_passes() {
        //Note: If in the string the character '#' is used, then delimit r### ... ###
        //with at least 1 # more than consecutive #-es in the string literal
        let op_str = r##"{"ops":[
        {"insert":"I am just a single sentence with "},
        {"attributes":{"bold":true},"insert":"bolded, "},
        {"attributes":{"italic":true},"insert":"italicized"},
        {"insert":", and "},
        {"attributes":{"underline":true},"insert":"underlined"},
        {"insert":" text which could be "},
        {"attributes":{"color":"red"},"insert":"a"},
        {"attributes":{"color":"#ed7d31"},"insert":"n"},
        {"attributes":{"color":"#70ad47"},"insert":"y"},
        {"insert":" "},
        {"attributes":{"color":"#2f5597"},"insert":"c"},
        {"attributes":{"color":"red"},"insert":"o"},
        {"attributes":{"color":"#ffc000"},"insert":"l"},
        {"attributes":{"color":"#00b0f0"},"insert":"o"},
        {"attributes":{"color":"#00b050"},"insert":"r "},
        {"insert":"you like, or it could even be "},
        {"attributes":{"underline":true,"italic":true,"bold":true},"insert":"all three"},
        {"insert":".\n"}]}"##;
        let aap: Delta = serde_json::from_str(op_str).unwrap();

        assert_eq!(aap.len(), 18);
    }
}
