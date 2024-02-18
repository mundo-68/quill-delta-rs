

#[test]
fn test_diff() {
    let a: &[usize] = &[0, 1, 2, 3, 4];
    let b: &[usize] = &[0, 1, 2, 9, 4];
    struct D {}
    impl diffs::Diff for D {
        type Error = ();
        fn delete(&mut self, o: usize, len: usize, new: usize) -> Result<(), ()> {
            assert_eq!(o, 3);
            assert_eq!(len, 1);
            assert_eq!(new, 3);
            println!("delete");
            Ok(())
        }
        fn insert(&mut self, o: usize, n: usize, len: usize) -> Result<(), ()> {
            assert_eq!(o, 3);
            assert_eq!(n, 3);
            assert_eq!(len, 1);
            println!("insert");
            Ok(())
        }
    }
    let mut diff = diffs::Replace::new(D {});
    diffs::myers::diff(&mut diff, a, 0, a.len(), b, 0, b.len()).unwrap()
}

#[test]
fn test_contiguous() {
    let a: &[usize] = &[0, 1, 2, 3, 4, 4, 4, 5];
    let b: &[usize] = &[0, 1, 2, 8, 9, 4, 4, 7];
    struct D {}
    impl diffs::Diff for D {
        type Error = ();
        fn delete(&mut self, _o: usize, _len: usize, _new: usize) -> Result<(), ()> {
            panic!("Should not delete")
        }
        fn insert(&mut self, _o: usize, _n: usize, _len: usize) -> Result<(), ()> {
            panic!("Should not insert")
        }
        fn replace(&mut self, o: usize, l: usize, n: usize, nl: usize) -> Result<(), ()> {
            assert!(o != 3 || (l == 2 && nl == 2));
            assert!(o != 7 || (l == 1 && nl == 1));
            println!("replace {:?} {:?} {:?} {:?}", o, l, n, nl);
            Ok(())
        }
    }
    let mut diff = diffs::Replace::new(D {});
    diffs::myers::diff(&mut diff, a, 0, a.len(), b, 0, b.len()).unwrap();
}

#[test]
fn test_replace() {
    let a: &[usize] = &[0, 1, 2, 3, 4];
    let b: &[usize] = &[0, 1, 2, 7, 8, 9];
    struct D {}
    impl diffs::Diff for D {
        type Error = ();
        fn delete(&mut self, _o: usize, _len: usize, _new: usize) -> Result<(), ()> {
            panic!("should not delete")
        }
        fn insert(&mut self, _o: usize, _n: usize, _len: usize) -> Result<(), ()> {
            panic!("should not insert")
        }
        fn replace(&mut self, _o: usize, _l: usize, _n: usize, _nl: usize) -> Result<(), ()> {
            Ok(())
        }
    }
    let mut diff = diffs::Replace::new(D {});
    diffs::myers::diff(&mut diff, a, 0, a.len(), b, 0, b.len()).unwrap();
}

#[test]
fn test_pat() {
    let a: &[usize] = &[0, 1, 3, 4, 5];
    let b: &[usize] = &[0, 1, 4, 5, 8, 9];
    struct D {}
    impl diffs::Diff for D {
        type Error = ();
        fn equal(&mut self, _o: usize, _len: usize, _new: usize) -> Result<(), ()> {
            println!("equal {:?} {:?} {:?}", _o, _len, _new);
            Ok(())
        }
        fn delete(&mut self, _o: usize, _len: usize, _new: usize) -> Result<(), ()> {
            println!("delete {:?} {:?} {:?}", _o, _len, _new);
            Ok(())
        }
        fn insert(&mut self, _o: usize, _n: usize, _len: usize) -> Result<(), ()> {
            println!("insert {:?} {:?} {:?}", _o, _n, _len);
            Ok(())
        }
        fn replace(&mut self, _o: usize, _l: usize, _n: usize, _nl: usize) -> Result<(), ()> {
            println!("replace {:?} {:?} {:?} {:?}", _o, _l, _n, _nl);
            Ok(())
        }
    }
    let mut diff = diffs::Replace::new(D {});
    diffs::myers::diff(&mut diff, a, 0, a.len(), b, 0, b.len()).unwrap();
}


