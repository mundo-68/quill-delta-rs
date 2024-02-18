use diffs::{myers, Diff, Replace};
use std::char;

#[test]
fn test_text() {
    let aa = "Hallo dit is een leuk verhaal";
    let a: Vec<char> = aa.chars().collect();
    let bb = "Hallo is een ander verhaal";
    let b: Vec<char> = bb.chars().collect();
    println!("{:?}", a);
    println!("{:?}", b);

    pub struct D {
        pub aaa: String,
        pub bbb: String,
        pub otn: String, //old to new
        pub nto: String, //new to old
    }

    impl Diff for D {
        type Error = ();
        fn equal(&mut self, _o: usize, _new: usize, _len: usize) -> Result<(), ()> {
            println!("equal {:?} {:?} {:?}", _o, _len, _new);
            dbg!(&self.aaa.to_string()[_o.._o + _len]);
            self.otn.push_str(&self.aaa.to_string()[_o.._o + _len]);
            self.nto.push_str(&self.bbb.to_string()[_new.._new + _len]);
            Ok(())
        }
        fn delete(&mut self, _o: usize, _len: usize, _new: usize) -> Result<(), ()> {
            println!("delete {:?} {:?} {:?}", _o, _len, _new);
            dbg!(&self.aaa.to_string()[_o.._o + _len]);
            self.nto.push_str(&self.aaa.to_string()[_o.._o + _len]);
            Ok(())
        }
        fn insert(&mut self, _o: usize, _n: usize, _len: usize) -> Result<(), ()> {
            println!("insert {:?} {:?} {:?}", _o, _n, _len);
            dbg!(&self.bbb.to_string()[_n.._n + _len]);
            self.otn.push_str(&self.bbb.to_string()[_n.._n + _len]);
            Ok(())
        }
    }
    let mut ddd: D = D {
        aaa: aa.to_string(),
        bbb: bb.to_string(),
        otn: String::new(),
        nto: String::new(),
    };
    let mut diff = Replace::new(&mut ddd);
    myers::diff(&mut diff, &a, 0, a.len(), &b, 0, b.len()).unwrap();
    dbg!(&ddd.otn);
    dbg!(&ddd.nto);
    assert_eq!(ddd.otn, bb);
    assert_eq!(ddd.nto, aa);
}
