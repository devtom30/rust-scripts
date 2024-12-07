use std::char;
use std::fs::File;
use std::fs::read_to_string;
use std::io::{BufWriter, Write};

use regex::Regex;

fn main() {
    let str_to_convert = String::from("GIORNO 3 - gioved&#xEC; 5 febbraio : PISA !!!");
    let new_str = convert_special_chars_in_str(&str_to_convert);
    println!("{}", new_str);

    let str_to_convert = String::from("met&#xE0");
    let new_str = convert_special_chars_in_str(&str_to_convert);
    println!("{}", new_str);

    let z = i64::from_str_radix("e9", 16);
    println!("{:?}", z);
    println!("{:?}", char::from_u32(z.unwrap() as u32));

    let c = convert_utf8_code_to_char(&"0xE0".to_string()).unwrap();
    println!("char converted: {}", c);

    let mut post_open = false;
    let re_post_aperture = Regex::new(r"<post><title>(.*)</title>").unwrap();
    let re_post_closure = Regex::new(r"</post>").unwrap();
    let mut vec_current_post: Vec<String> = vec!();
    let mut current_post_number = 0;
    let mut current_post_title: String = String::from("");

    for line in read_to_string(
        "/home/tom/.config/JetBrains/RustRover2024.1/scratches/scratch_2.xml")
        .expect("unable to open file for read")
        .lines() {
        let line = line.replace("<![CDATA[", "");
        let line = line.replace("]]>", "");
        let line = line.as_str();
        if post_open {
            vec_current_post.push(line.parse().unwrap());
            if re_post_closure.is_match(line) {
                post_open = false;
                let converted_title = convert_special_chars_in_str(&current_post_title);
                write_vec_to_file(current_post_number, vec_current_post, &converted_title);
                vec_current_post = vec!();
            }
        } else {
            if let Some(caps) = re_post_aperture.captures(line) {
                post_open = true;
                current_post_number += 1;
                current_post_title = String::from_utf8(Vec::from(caps.get(1).map_or("", |m| m.as_str()).as_bytes())).unwrap();
                vec_current_post.push(line.parse().unwrap());
            }
        }
    }
}

fn convert_special_chars_in_str(str: &String) -> String {
    let mut take_special_char = false;
    let mut special_char_code_str: String = String::from("");
    let mut str_ok = String::from("");
    for c in str.chars() {
        if c == '&' {
            take_special_char = true;
            special_char_code_str.push(c);
        } else {
            if c == ';' && take_special_char {
                take_special_char = false;
                let converted = convert_utf8_code_to_char(&special_char_code_str).expect("can't convert char code");
                println!("converted {}", converted);
                str_ok.push(converted);
                println!("new str with special char: {}", str_ok);
                special_char_code_str = String::from("");
            } else {
                if take_special_char {
                    special_char_code_str.push(c);
                } else {
                    str_ok.push(c);
                }
            }
        }
    }
    str_ok.replace("/", "|")
}

fn write_vec_to_file(nb: i32, vector: Vec<String>, title: &String) {
    let file_name = format!("out/post_{number:0>3}__{title}.html", number=nb, title=title);
    let file = File::create(&file_name).expect(format!("can't create file {}", file_name).as_str());
    let mut file = BufWriter::new(file);

    writeln!(file, "<html>").unwrap();
    writeln!(file, "<head><title>{}</title><meta charset=\"utf-8\" /></head>", title).unwrap();
    writeln!(file, "<body>").unwrap();

    for value in vector {
        writeln!(file, "{}", value).unwrap();
    }

    writeln!(file, "</body>").unwrap();
    writeln!(file, "</html>").unwrap();

    file.flush().unwrap();
}

fn convert_utf8_code_to_char(raw: &String) -> Option<char> {
    println!("converting: {}", raw);
    let mut without_prefix= "";
    let prefixes = ["0x", "&#x"];
    for prefix in prefixes {
        if raw.starts_with(prefix) {
            without_prefix = raw.trim_start_matches(prefix);
            break;
        }
    }
    println!("converting cleaned {}", without_prefix);
    let z = i64::from_str_radix(without_prefix, 16);
    char::from_u32(z.unwrap() as u32)
}
