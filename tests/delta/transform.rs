#[cfg(test)]
mod tests {
    use anyhow::Result;
    use delta::attributes::Attributes;
    use delta::delta::Delta;
    use delta::optransform::OpTransform;

    #[test]
    fn compose_insert_and_insert_passes() -> Result<()> {
        let mut a1 = Delta::default();
        a1.insert("A");
        let a2 = a1.clone();
        let mut b1 = Delta::default();
        b1.insert("B");

        let b2 = b1.clone();

        let mut expected1 = Delta::default();
        expected1.retain(1);
        expected1.insert("B");

        let mut expected2 = Delta::default();
        expected2.insert("B");

        let r = a1.transform(&b1, true)?;
        assert_eq!(&r, &expected1);
        let r = a2.transform(&b2, false)?;
        assert_eq!(&r, &expected2);
        Ok(())
    }

    #[test]
    fn compose_insert_and_retain_passes() -> Result<()> {
        let mut a = Delta::default();
        a.insert("A");

        let mut attr = Attributes::default();
        attr.insert("bold", true);
        attr.insert("color", "red");

        let mut b = Delta::default();
        b.retain_attr(1, attr.clone());

        let mut expected = Delta::default();
        expected.retain(1);
        expected.retain_attr(1, attr.clone());

        let r = a.transform(&b, false)?;
        assert_eq!(&r, &expected);

        //added by me
        let r = a.transform(&b, true)?;
        assert_eq!(&r, &expected);
        Ok(())
    }

    #[test]
    fn compose_insert_and_delete_passes() -> Result<()> {
        let mut a = Delta::default();
        a.insert("A");

        let mut b = Delta::default();
        b.delete(1);

        let mut expected = Delta::default();
        expected.retain(1);
        expected.delete(1);

        let r = a.transform(&b, false)?;
        assert_eq!(&r, &expected);

        //added by me
        let r = a.transform(&b, true)?;
        assert_eq!(&r, &expected);
        Ok(())
    }

    #[test]
    fn compose_delete_insert_passes() -> Result<()> {
        let mut a = Delta::default();
        a.delete(1);

        let mut b = Delta::default();
        b.insert("B");

        let mut expected = Delta::default();
        expected.insert("B");

        let r = a.transform(&b, false)?;
        assert_eq!(&r, &expected);

        //added by me
        let r = a.transform(&b, true)?;
        assert_eq!(&r, &expected);
        Ok(())
    }

    #[test]
    fn compose_delete_retain_passes() -> Result<()> {
        let mut a = Delta::default();
        a.delete(1);

        let mut attr = Attributes::default();
        attr.insert("bold", true);
        attr.insert("color", "red");

        let mut b = Delta::default();
        b.retain_attr(1, attr.clone());

        let expected = Delta::default();

        let r = a.transform(&b, true)?;
        assert_eq!(&r, &expected);
        Ok(())
    }

    #[test]
    fn compose_delete_delete_passes() -> Result<()> {
        let mut a = Delta::default();
        a.delete(1);

        let mut b = Delta::default();
        b.delete(1);

        let expected = Delta::default();

        let r = a.transform(&b, false)?;
        assert_eq!(&r, &expected);
        Ok(())
    }

    #[test]
    fn compose_retain_insert_passes() -> Result<()> {
        let mut attr = Attributes::default();
        attr.insert("color", "blue");

        let mut a = Delta::default();
        a.retain_attr(1, attr);

        let mut b = Delta::default();
        b.insert("B");

        let mut expected = Delta::default();
        expected.insert("B");

        let r = a.transform(&b, true)?;
        assert_eq!(&r, &expected);
        Ok(())
    }

    #[test]
    fn compose_retain_retain_passes() -> Result<()> {
        let mut blue = Attributes::default();
        blue.insert("color", "blue");

        let mut bold_red = Attributes::default();
        bold_red.insert("bold", true);
        bold_red.insert("color", "red");

        let mut bold = Attributes::default();
        bold.insert("bold", true);

        let mut a1 = Delta::default();
        a1.retain_attr(1, blue.clone());

        let a2 = a1.clone();
        let mut b1 = Delta::default();
        b1.retain_attr(1, bold_red.clone());

        let b2 = b1.clone();

        let mut expected1 = Delta::default();
        expected1.retain_attr(1, bold);

        let expected2 = Delta::default();

        let r = a1.transform(&b1, true)?;
        assert_eq!(&r, &expected1);
        let r = b2.transform(&a2, true)?;
        assert_eq!(&r, &expected2);
        Ok(())
    }

    #[test]
    fn compose_retain_retain_without_prio_passes() -> Result<()> {
        let mut blue = Attributes::default();
        blue.insert("color", "blue");

        let mut bold_red = Attributes::default();
        bold_red.insert("bold", true);
        bold_red.insert("color", "red");

        let mut a1 = Delta::default();
        a1.retain_attr(1, blue.clone());

        let a2 = a1.clone();
        let mut b1 = Delta::default();
        b1.retain_attr(1, bold_red.clone());

        let b2 = b1.clone();

        let mut expected1 = Delta::default();
        expected1.retain_attr(1, bold_red);

        let mut expected2 = Delta::default();
        expected2.retain_attr(1, blue);

        let r = a1.transform(&b1, false)?;
        assert_eq!(&r, &expected1);
        let r = b2.transform(&a2, false)?;
        assert_eq!(&r, &expected2);
        Ok(())
    }

    #[test]
    fn compose_retain_delete_passes() -> Result<()> {
        let mut attr = Attributes::default();
        attr.insert("color", "blue");

        let mut a = Delta::default();
        a.retain_attr(1, attr);

        let mut b = Delta::default();
        b.delete(1);

        let mut expected = Delta::default();
        expected.delete(1);

        let r = a.transform(&b, true)?;
        assert_eq!(&r, &expected);
        Ok(())
    }

    #[test]
    fn compose_alternating_edits_passes() -> Result<()> {
        let mut a1 = Delta::default();
        a1.retain(2);
        a1.insert("si");
        a1.delete(5);

        let a2 = a1.clone();
        let mut b1 = Delta::default();
        b1.retain(1);
        b1.insert("e");
        b1.delete(5);
        b1.retain(1);
        b1.insert("ow");

        let b2 = b1.clone();
        let mut expected1 = Delta::default();
        expected1.retain(1);
        expected1.insert("e");
        expected1.delete(1);
        expected1.retain(2);
        expected1.insert("ow");

        let mut expected2 = Delta::default();
        expected2.retain(2);
        expected2.insert("si");
        expected2.delete(1);

        let r = a1.transform(&b1, false)?;
        assert_eq!(&r, &expected1);
        let r = b2.transform(&a2, false)?;
        assert_eq!(&r, &expected2);
        Ok(())
    }

    #[test]
    fn compose_conflicting_appends_passes() -> Result<()> {
        let mut a1 = Delta::default();
        a1.retain(3);
        a1.insert("aa");

        let a2 = a1.clone();
        let mut b1 = Delta::default();
        b1.retain(3);
        b1.insert("bb");

        let b2 = b1.clone();
        let mut expected1 = Delta::default();
        expected1.retain(3);
        expected1.insert("bb");

        let mut expected2 = Delta::default();
        expected2.retain(3);
        expected2.insert("aa");

        let r = a1.transform(&b1, false)?;
        assert_eq!(&r, &expected1);
        let r = b2.transform(&a2, false)?;
        assert_eq!(&r, &expected2);
        Ok(())
    }

    #[test]
    fn compose_prepend_appends_passes() -> Result<()> {
        let mut a1 = Delta::default();
        a1.insert("aa");

        let a2 = a1.clone();
        let mut b1 = Delta::default();
        b1.retain(3);
        b1.insert("bb");

        let b2 = b1.clone();
        let mut expected1 = Delta::default();
        expected1.retain(5);
        expected1.insert("bb");

        let mut expected2 = Delta::default();
        expected2.insert("aa");

        let r = a1.transform(&b1, false)?;
        assert_eq!(&r, &expected1);
        let r = b2.transform(&a2, false)?;
        assert_eq!(&r, &expected2);
        Ok(())
    }

    #[test]
    fn compose_trailing_deletes_with_different_length_passes() -> Result<()> {
        let mut a1 = Delta::default();
        a1.retain(2);
        a1.delete(1);

        let a2 = a1.clone();
        let mut b1 = Delta::default();
        b1.delete(3);

        let b2 = b1.clone();
        let mut expected1 = Delta::default();
        expected1.delete(2);

        let expected2 = Delta::default();

        let r = a1.transform(&b1, false)?;
        assert_eq!(&r, &expected1);
        let r = b2.transform(&a2, false)?;
        assert_eq!(&r, &expected2);
        Ok(())
    }

    #[test]
    fn compose_immutability_passes() -> Result<()> {
        let mut a1 = Delta::default();
        a1.insert("A");

        let a2 = a1.clone();
        let mut b1 = Delta::default();
        b1.insert("B");

        let b2 = b1.clone();
        let mut expected = Delta::default();
        expected.retain(1);
        expected.insert("B");

        let r = a1.transform(&b1, true)?;
        assert_eq!(&r, &expected);
        assert_eq!(a1, a2);
        assert_eq!(b1, b2);
        Ok(())
    }
}
