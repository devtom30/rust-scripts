use std::fs;
use std::fs::read_to_string;

fn main() {
    let list_file = "/home/tom/RustroverProjects/scripts/links.html";

    let paths = fs::read_dir("/home/tom/Downloads/blog_articles_proteges_mot_de_passe")
        .unwrap();

    let titles: Vec<String> = paths.map(|path| path.unwrap().file_name())
        .map(|file_name| file_name.to_string_lossy().split(" ").next().unwrap().to_string())
        .collect();

    println!("{:?}", titles);

    let mut not_found: Vec<String> = vec!();
    let mut nb_found: u64 = 0;
    for line in read_to_string(list_file)
        .expect("unable to open file for read")
        .lines() {
        let title = line.split("/").last().unwrap();
        let mut found = titles.contains(&title.to_string());
        println!("title {} found? {}", title, &found);
        if !found {
            &mut not_found.push(title.to_string() + " : " + line);
        } else {
            nb_found += 1;
        }
    }

    println!("found {}", nb_found.to_string());
    println!("not found {}", &not_found.len());
    not_found.iter().for_each(|string: &String| println!("{string}"));
}
