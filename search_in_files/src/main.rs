use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::Path;
use app_properties::AppProperties;
use grep_matcher::Matcher;
use grep_regex::RegexMatcher;
use grep_searcher::Searcher;
use grep_searcher::sinks::UTF8;
use regex::Regex;
use walkdir::WalkDir;

fn main() {
    let regex: Vec<Regex> = ["facebook", "twitter", "apiLocator"].to_vec()
        .iter().map(|str| Regex::new(str).expect("regex"))
        .collect();
    assert!(search_exclusions_line(&String::from("auieau Facebook"), &regex));

    let args: Vec<String> = env::args().collect();
    let pattern = &args[1];

    let properties: AppProperties = AppProperties::new();
    let path = properties.get("path");
    let filename_pattern = properties.get("filename_pattern");
    let exclusions_str = properties.get("exclusions");
    let exclusion_map = extract_exclusions(exclusions_str);

    search_pattern_at_path(Path::new(path), pattern, &filename_pattern.to_string(), &exclusion_map);
}

struct SearchResult {
    path: String,
    href: String,
    lines: Vec<String>
}

fn extract_exclusions(value: &str) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let mut parts: Vec<String> = value.split(',')
        .collect::<Vec<&str>>()
        .into_iter()
        .map(|str| str.to_string())
        .collect();
    println!("{:?}", parts[1..].to_vec());
    map.insert(parts.get(0).expect("key").to_string(), parts[1..].to_vec());
    map
}

fn search_pattern_at_path(path: &Path, pattern: &String, filename_pattern: &String,
                          exclusion_patterns: &HashMap<String, Vec<String>>) -> Vec<SearchResult> {
    let mut results = vec!();
    search_in_dir(&mut results, pattern, path.to_string_lossy().to_string(), &filename_pattern,
                  exclusion_patterns);
    results
}

fn search_in_dir(results: &mut Vec<SearchResult>, pattern: &String, current_dir: String,
                 filename_pattern: &String, exclusion_patterns: &HashMap<String, Vec<String>>) {
    let matcher = RegexMatcher::new(pattern.to_lowercase().as_str()).expect("regex");

    let mut exclusion_regex_matchers: Vec<Regex> = vec!();

    if exclusion_patterns.contains_key(pattern) {
        exclusion_regex_matchers = exclusion_patterns.get(pattern)
            .expect("pattern")
            .iter()
            .map(|exclusion_pattern| {
                Regex::new(exclusion_pattern).expect("exclusion pattern matcher build")
            }).collect();
    }


    let mut nb = 0;
    for entry in WalkDir::new(current_dir.clone())
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(filename_pattern)) {
        let f_name = entry.file_name().to_string_lossy();
        let path = entry.path().to_string_lossy().to_string();
        let rel_path= make_path_relative(&current_dir, &path);

        nb += 1;
        println!("{} search {} in file {}", nb, pattern, entry.path().to_string_lossy().to_string());
        let lines = search_in_file(&matcher, entry.path(), &exclusion_regex_matchers);
        if !lines.is_empty() {
            results.push(SearchResult {
                path: path,
                href: rel_path,
                lines
            });
        }
    }
}

fn make_path_relative(start_path: &String, path_to_shorten: &String) -> String {
    let path_to_shorten_length = path_to_shorten.len();
    path_to_shorten.chars().skip(start_path.len()).take(path_to_shorten_length - start_path.len()).collect()
}

fn search_in_file(matcher: &RegexMatcher, path: &Path,
                  exclusion_regex: &Vec<Regex>) -> Vec<String> {
    let file = File::open(&path).expect("open file");
    let mut matches: Vec<(u64, String)> = vec![];
    Searcher::new().search_file(&matcher, &file, UTF8(|lnum, line| {
        // We are guaranteed to find a match, so the unwrap is OK.
        let mymatch = matcher.find(line.to_lowercase().as_bytes())?.unwrap();

        let mut exclusion_found = false;

        if !exclusion_found {
            matches.push((lnum, line.to_string()));
        } else {
            print!("line excluded {}", line.to_string());
        }
        Ok(true)
    })).expect("search_slice");

    return matches.into_iter().map(|(lnum, line)| line).collect()
}

fn search_exclusions_line(line: &String, exclusion_regex: &Vec<Regex>) -> bool {
    let line_lower_case = line.clone().to_lowercase();
    let mut exclusion_found = false;
    for regex_matcher in exclusion_regex {
        if regex_matcher.is_match(line_lower_case.as_ref()) {
            println!("match {}", regex_matcher.to_string());
            println!("{}", line_lower_case);
            exclusion_found = true;
        } else {
            println!("no match {}", regex_matcher.to_string());
            println!("{}", line_lower_case)
        }
    }
    exclusion_found
}
