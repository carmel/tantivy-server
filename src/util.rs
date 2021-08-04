use std::{ops::Range, str};

pub struct Util;

impl Util {
    pub fn fragment_boundary_cut(s: &str, mut range: Range<usize>) -> &str {
        if s.len() <= range.end {
            range.end = s.len();
        } else {
            while !s.is_char_boundary(range.end) {
                range.end += 1;
            }
        }

        if range.start != 0 {
            while !s.is_char_boundary(range.start) {
                range.start -= 1;
            }
        }

        &s[range]
    }
}

#[test]
fn test_util() {
    // let msg = Util::encode(Status::Ok, serde_json::to_value("hello world!").unwrap());
    // println!("{}", str::from_utf8(&msg[..]).unwrap());
    let str = "圣经是什么？我们知道，“圣经”这辞的意思是“那书。”但这书是什么？圣经本身说，“圣经都是神的呼出。”";
    println!("{:?}", Util::fragment_boundary_cut(str, 0..3));
    println!("{:?}", "hello".as_bytes());

    if cfg!(target_endian = "big") {
        println!("Big endian");
    } else {
        println!("Little endian");
    }

    println!("{}", cfg!(target_os));
}
