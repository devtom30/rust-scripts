use blake2::{Blake2b512, Digest};
use chrono::offset::Local;
use chrono::Datelike;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    remove_duplicates("/media/tom/Toshiba4ToCanvio/.save")
}

pub fn remove_duplicates(dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut size_map: HashMap<u64, Vec<String>> = std::collections::HashMap::new();
    let mut blake2map: HashMap<String, String> = std::collections::HashMap::new();
    let mut duplicates: BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
    let mut nb_duplicates: u64 = 0;

    let _ = WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().ok().map_or(false, |m| m.is_file()))
        .filter(|e| !e.path().display().to_string().ends_with(".rar"))
        .for_each(|f| {
            let size = f.metadata().unwrap().len();
            let path = f.path().display().to_string();
            println!("{} {}", size, path);
            if size_map.contains_key(&size) {
                // must compute blake2 of this file
                let blake2_hash = compute_blake2(path.as_str());
                if blake2_hash.is_empty() {
                    return ;
                }
                blake2map.insert(path.clone(), blake2_hash);
                for v in size_map.get(&size).unwrap() {
                    let blake2_hash_other: String = match blake2map.get(v) {
                        None => { compute_blake2(v.as_str()) }
                        Some(hash) => { hash.clone() }
                    };
                    let blake2_hash = blake2map.get(&path).unwrap();
                    if blake2_hash_other.eq(blake2_hash) {
                        if duplicates.get(blake2_hash).is_none() {
                            duplicates.entry(blake2_hash.clone()).or_insert(vec![v.clone(), path.clone()]);
                        } else {
                            duplicates.get_mut(blake2_hash).unwrap().push(path.clone());
                        }
                        println!("duplicate: {} {}", v, path);
                        nb_duplicates += 1;
                        break;
                    }
                }
                size_map.get_mut(&size).unwrap_or(&mut vec!()).push(path);
            } else {
                size_map.insert(size, vec![path]);
            }
        });

    let now = Local::now().format("%Y-%m-%d_%H:%M:%S").to_string();
    let remove_script_path = prepare_script_to_remove_duplicates(&duplicates, &now);

    println!("{nb_duplicates} duplicates:");
    duplicates.into_iter().for_each(|(k, v)| {
        println!("{}", k);
        v.into_iter().for_each(|x| {
            println!("    {}", x);
        });
    });

    println!("remove script: {}", remove_script_path);

    Ok(())
}

fn compute_blake2(path: &str) -> String {
    let mut hasher = Blake2b512::new();
    let contents = fs::read(path).unwrap_or_else(|e| {
        println!("couldn't read file: {}", e);
        vec![]
    });
    hasher.update(contents);
    let hash = hasher.finalize();
    let hash_str = format!("{:x}", hash);
    hash_str
}

fn prepare_script_to_remove_duplicates(map: &BTreeMap<String, Vec<String>>, date_time: &String) -> String {
    let path = "remove_duplicates_".to_owned() + date_time.as_str() + ".sh";
    let mut output = File::create(&path).expect("can't create the output file");
    map.into_iter().for_each(|(_k, v)| {
        v.iter().skip(1).for_each(|x| {
            let line = format!("rm {}", x);
            writeln!(output, "{}", line).expect("can't write line to file");
        });
    });
    path
}

#[cfg(test)]
mod tests {
    use crate::{prepare_script_to_remove_duplicates, remove_duplicates};
    use chrono::Local;
    use std::collections::{BTreeMap, HashMap};
    use std::fs;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;

    #[test]
    fn remove_duplicates_test() {
        let dir = "/home/tom/RustroverProjects/scripts/remove_duplicates/resources/test/root_directory";
        remove_duplicates(dir).expect("Error");
    }

    #[test]
    fn prepare_script_to_remove_duplicates_test() {
        let map: BTreeMap<String, Vec<String>> = std::collections::BTreeMap::from([
            ("hash1".to_string(), vec!["file1".to_string(), "file2".to_string(), "file3".to_string()]),
            ("hash2".to_string(), vec!["file4".to_string(), "file5".to_string(), "file6".to_string()]),
            ("hash3".to_string(), vec!["file7".to_string(), "file8".to_string()])
        ]);
        let now = Local::now().format("%Y-%m-%d_%H:%M:%S").to_string();
        let now = "2025-01-01_00:00:00".to_string();
        let path = prepare_script_to_remove_duplicates(&map, &now);
        assert_eq!(path, "remove_duplicates_2025-01-01_00:00:00.sh");

        let path = Path::new(path.as_str());
        let display = path.display();
        let path_tmp_str = path.to_str().unwrap().to_string() + ".tmp";
        let path_tmp = Path::new(&path_tmp_str);
        fs::copy(path, path_tmp).unwrap_or_else(|_| panic!("couldn't copy file"));
        fs::remove_file(path).unwrap_or_else(|_| println!("can't remove file, maybe it doesn't exist"));
        let file = match File::open(&path_tmp) {
            Err(why) => panic!("couldn't open {}: {}", display, why.to_string()),
            Ok(file) => file,
        };
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines()
            .filter_map(|l| l.ok())
            .collect();
        assert_eq!(
            lines.get(0).unwrap_or_else(||panic!("no line")),
            "rm file2"
        );
        assert_eq!(
            lines.get(1).unwrap_or_else(||panic!("no line")),
            "rm file3"
        );
        assert_eq!(
            lines.get(2).unwrap_or_else(||panic!("no line")),
            "rm file5"
        );
        assert_eq!(
            lines.get(3).unwrap_or_else(||panic!("no line")),
            "rm file6"
        );
        assert_eq!(
            lines.get(4).unwrap_or_else(||panic!("no line")),
            "rm file8"
        );
    }
}