#![feature(test)]

extern crate test;

use rayon::prelude::*;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::{self, Read};

const PATH: &str = "assets/1e5.sql";

#[bench]
fn bench_read_to_string(b: &mut test::Bencher) {
    b.iter(|| {
        fs::read_to_string(PATH).unwrap();
    });
}

#[bench]
fn bench_to_vec(b: &mut test::Bencher) {
    b.iter(move || {
        let s = fs::read_to_string(PATH).unwrap();
        let _buf: Vec<Vec<char>> = s.lines().map(|l| l.trim_end().chars().collect()).collect();
    });
}

#[bench]
fn bench_to_vec_rayon(b: &mut test::Bencher) {
    b.iter(move || {
        let s = fs::read_to_string(PATH).unwrap();
        let _buf: Vec<Vec<char>> = s
            .lines()
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|l| l.trim_end().chars().collect())
            .collect();
    });
}

#[bench]
fn bench_to_vec_read_line(b: &mut test::Bencher) {
    b.iter(move || {
        let f = fs::File::open(PATH).unwrap();
        let mut reader = BufReader::new(f);
        let mut v: Vec<Vec<char>> = Vec::new();
        let mut buf = String::new();

        while {
            buf.clear();
            reader.read_line(&mut buf).unwrap() != 0
        } {
            v.push(buf.trim_end().chars().collect());
        }
    });
}

#[bench]
fn bench_read_to_string_and_ropey(b: &mut test::Bencher) {
    b.iter(move || {
        let s = fs::read_to_string(PATH).unwrap();
        ropey::Rope::from(s);
    });
}

// From @ubnt_intrepid below

macro_rules! try_opt_result {
    ($e:expr) => {
        match $e? {
            Ok(b) => b,
            Err(e) => return Some(Err(e)),
        }
    };
}

struct Lines<R> {
    bytes: io::Bytes<R>,
}

impl<R> Lines<R>
where
    R: Read,
{
    #[inline]
    fn to_utf8_char(&mut self, buf: &mut [u8; 4], remaining: usize) -> Option<io::Result<char>> {
        for i in 0..remaining {
            buf[i + 1] = try_opt_result!(self.bytes.next());
        }
        let s = std::str::from_utf8(&buf[..remaining + 1]).expect("invalid utf8 character");
        let c = s.parse().expect("faild to parse str to char");
        Some(Ok(c))
    }

    #[inline]
    fn next_char(&mut self) -> Option<io::Result<char>> {
        let mut buf = [0u8; 4];
        buf[0] = try_opt_result!(self.bytes.next());
        match buf[0] {
            0b_0000_0000...0b_0111_1111 => Some(Ok(buf[0] as char)),
            0b_1100_0000...0b_1101_1111 => self.to_utf8_char(&mut buf, 1),
            0b_1110_0000...0b_1110_1111 => self.to_utf8_char(&mut buf, 2),
            0b_1111_0000...0b_1111_0111 => self.to_utf8_char(&mut buf, 3),
            _ => panic!("invalid bit pattern"),
        }
    }
}

impl<R> Iterator for Lines<R>
where
    R: Read,
{
    type Item = io::Result<Vec<char>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf: Option<Vec<char>> = None;
        while let Some(c) = self.next_char() {
            let c = match c {
                Ok(c) => c,
                Err(e) => return Some(Err(e)),
            };
            buf.get_or_insert_with(Default::default).push(c);
            if c == '\n' {
                break;
            }
        }
        buf.map(Ok)
    }
}

fn main() {
    let s = "abc\nあいうえお\n";

    let v: Vec<Vec<char>> = Lines {
        bytes: io::Cursor::new(s).bytes(),
    }
    .collect::<Result<_, _>>()
    .unwrap();

    println!("{:?}", v);
}

#[bench]
fn bench_to_vec_char_iter(b: &mut test::Bencher) {
    b.iter(|| {
        let f = fs::File::open(PATH).unwrap();
        let reader = BufReader::new(f);
        let v: Vec<Vec<char>> = Lines {
            bytes: reader.bytes(),
        }
        .collect::<Result<_, _>>()
        .unwrap();
        println!("{}", v.len());
    });
}
