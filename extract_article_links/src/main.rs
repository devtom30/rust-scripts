use std::fs::File;
use std::io::{BufReader, Write};
use utf8_chars::BufReadCharsExt;
use regex::Regex;

fn main() {
    let mut f = BufReader::new(File::open("/home/tom/angela/blog/page_web_articles_proteges_mot_de_passe/Eklablog.html")
        .expect("open failed"));

    let mut out = File::create("result.html").expect("create file");

    const LT_CHAR: char = '<';
    let mut lt_char_last = false;
    let mut tag_being_extracted = String::from("");
    let mut current_line = String::from("");
    for c in f.chars().map(|x| x.unwrap()) {
        if c == LT_CHAR {
            write!(out, "{}", current_line).expect("write");
            write!(out, "\n").expect("write");
            current_line = String::from("");
        }
        current_line.push(c);
    }

    write!(out, "{}", current_line).expect("write");
}
