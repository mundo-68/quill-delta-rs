#[cfg(test)]
mod tests {
    use delta::delta::Delta;
    use delta::optransform::OpTransform;

    #[test]
    fn transform_insert_before_position_passes() -> anyhow::Result<()> {
        let mut a = Delta::default();
        a.insert("A");

        let r = a.transform_position(2, false)?;
        assert_eq!(r, 3);
        Ok(())
    }

    #[test]
    fn transform_insert_after_position_passes() -> anyhow::Result<()> {
        let mut a = Delta::default();
        a.retain(2);
        a.insert("A");

        let r = a.transform_position(1, false)?;
        assert_eq!(r, 1);
        Ok(())
    }

    #[test]
    fn transform_insert_at_position_passes() -> anyhow::Result<()> {
        let mut a = Delta::default();
        a.retain(2);
        a.insert("A");

        let r = a.transform_position(2, true)?;
        assert_eq!(r, 2);

        let r = a.transform_position(2, false)?;
        assert_eq!(r, 3);
        Ok(())
    }

    #[test]
    fn transform_delete_before_position_passes() -> anyhow::Result<()> {
        let mut a = Delta::default();
        a.delete(2);

        let r = a.transform_position(4, false)?;
        assert_eq!(r, 2);
        Ok(())
    }

    #[test]
    fn transform_delete_after_position_passes() -> anyhow::Result<()> {
        let mut a = Delta::default();
        a.retain(4);
        a.delete(2);

        let r = a.transform_position(2, false)?;
        assert_eq!(r, 2);
        Ok(())
    }

    #[test]
    fn transform_delete_across_position_passes() -> anyhow::Result<()> {
        let mut a = Delta::default();
        a.retain(1);
        a.delete(4);

        let r = a.transform_position(2, false)?;
        assert_eq!(r, 1);
        Ok(())
    }

    #[test]
    fn transform_insert_and_delete_before_position_passes() -> anyhow::Result<()> {
        let mut a = Delta::default();
        a.retain(2);
        a.insert("A");
        a.delete(2);

        let r = a.transform_position(4, false)?;
        assert_eq!(r, 3);
        Ok(())
    }

    #[test]
    fn transform_insert_beforre_and_delete_across_position_passes() -> anyhow::Result<()> {
        let mut a = Delta::default();
        a.retain(2);
        a.insert("A");
        a.delete(4);

        let r = a.transform_position(4, false)?;
        assert_eq!(r, 3);
        Ok(())
    }

    #[test]
    fn transform_delete_beforre_and_delete_across_position_passes() -> anyhow::Result<()> {
        let mut a = Delta::default();
        a.delete(1);
        a.retain(1);
        a.delete(4);

        let r = a.transform_position(4, false)?;
        assert_eq!(r, 1);
        Ok(())
    }
}
