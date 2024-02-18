#[cfg(test)]
mod tests {
    use anyhow::Result;
    use delta::attributes::Attributes;
    use delta::delta::Delta;
    use delta::document::Document;
    use delta::optransform::OpTransform;
    use delta::types::attr_val::AttrVal;

    #[test]
    fn invert_insert_passes() -> Result<()> {
        let mut delta = Delta::default();
        delta.retain(2);
        delta.insert("A");

        let mut base = Delta::default();
        base.insert("123456");

        let mut expected = Delta::default();
        expected.retain(2);
        expected.delete(1);

        let inverted = delta.invert(&base);
        assert_eq!(&inverted, &expected);
        let res = base.compose(&delta)?.compose(&inverted)?;
        assert_eq!(&res, &base);
        Ok(())
    }

    #[test]
    fn invert_delete_passes() -> Result<()> {
        let mut delta = Delta::default();
        delta.retain(2);
        delta.delete(3);

        let mut base = Delta::default();
        base.insert("123456");

        let mut expected = Delta::default();
        expected.retain(2);
        expected.insert("345");

        let inverted = delta.invert(&base);
        assert_eq!(&inverted, &expected);

        let res = base.compose(&delta)?.compose(&inverted)?;
        assert_eq!(&res, &base);
        Ok(())
    }

    #[test]
    fn invert_retain_passes() -> Result<()> {
        let mut attr = Attributes::default();
        attr.insert("bold", true);

        let mut delta = Delta::default();
        delta.retain(2);
        delta.retain_attr(3, attr);

        let mut base = Delta::default();
        base.insert("123456");

        let mut attr = Attributes::default();
        attr.insert("bold", AttrVal::Null);
        let mut expected = Delta::default();
        expected.retain(2);
        expected.retain_attr(3, attr);

        let inverted = delta.invert(&base);
        assert_eq!(&inverted, &expected);

        let res = base.compose(&delta)?.compose(&inverted)?;
        assert_eq!(&res, &base);
        Ok(())
    }

    #[test]
    fn invert_retain_on_a_delta_with_different_attributes_passes() -> Result<()> {
        let mut attr = Attributes::default();
        attr.insert("bold", true);
        let mut base = Delta::default();
        base.insert("123");
        base.insert_attr(4, attr);

        let mut attr = Attributes::default();
        attr.insert("italic", true);
        let mut delta = Delta::default();
        delta.retain_attr(4, attr);

        let mut attr = Attributes::default();
        attr.insert("italic", AttrVal::Null);
        let mut expected = Delta::default();
        expected.retain_attr(4, attr);

        let inverted = delta.invert(&base);
        assert_eq!(&inverted, &expected);

        let res = base.compose(&delta)?.compose(&inverted)?;
        assert_eq!(&res, &base);
        Ok(())
    }

    #[test]
    fn invert_combined_passes() -> Result<()> {
        let mut bold = Attributes::default();
        bold.insert("bold", true);
        let mut italic = Attributes::default();
        italic.insert("italic", true);
        let mut inbt = Attributes::default();
        inbt.insert("italic", AttrVal::Null);
        inbt.insert("bold", true);
        let mut itbn = Attributes::default();
        itbn.insert("italic", true);
        itbn.insert("bold", AttrVal::Null);
        let mut red = Attributes::default();
        red.insert("color", "red");
        let mut red_bold = Attributes::default();
        red_bold.insert("color", "red");
        red_bold.insert("bold", true);

        let mut delta = Delta::default();
        delta.retain(2);
        delta.delete(2);
        delta.insert_attr("AB", italic.clone());
        delta.retain_attr(2, inbt.clone());
        delta.retain_attr(2, red.clone());
        delta.delete(1);

        let mut base = Delta::default();
        base.insert_attr("123", bold.clone());
        base.insert_attr("456", italic.clone());
        base.insert_attr("789", red_bold.clone());

        let mut expected = Delta::default();
        expected.retain(2);
        expected.insert_attr("3", bold.clone());
        expected.insert_attr("4", italic.clone());
        expected.delete(2);
        expected.retain_attr(2, itbn.clone());
        expected.retain(2);
        expected.insert_attr("9", red_bold.clone());

        let inverted = delta.invert(&base);
        assert_eq!(&inverted, &expected);

        let res = base.compose(&delta)?.compose(&inverted)?;
        assert_eq!(&res, &base);
        Ok(())
    }
}
