#![feature(test)]

extern crate test;

use std::fs;
use std::io::BufRead;
use std::io::BufReader;

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
