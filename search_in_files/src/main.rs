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
    let download = args.get(2).map_or_else(|| false, |arg| true);
    println!("{}", download);

    let properties: AppProperties = AppProperties::new();
    let path = properties.get("path");
    let filename_pattern = properties.get("filename_pattern");
    let exclusions_str = properties.get("exclusions");
    let exclusion_map = extract_exclusions(exclusions_str);

    search_pattern_at_path(Path::new(path), pattern, &filename_pattern.to_string(), &exclusion_map,
    download);
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
                          exclusion_patterns: &HashMap<String, Vec<String>>,
                          download: bool) -> Vec<SearchResult> {
    let mut results = vec!();
    search_in_dir(&mut results, pattern, path.to_string_lossy().to_string(), &filename_pattern,
                  exclusion_patterns, download);
    results
}

fn search_in_dir(results: &mut Vec<SearchResult>, pattern: &String, current_dir: String,
                 filename_pattern: &String, exclusion_patterns: &HashMap<String, Vec<String>>,
                 download: bool) {
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
        println!("found {}", lines.len());
        if !lines.is_empty() {
            if download {
                // let's extract links and download them
                let regexes: Vec<Regex> = to_vec_regex(&vec!(String::from("ekladata")));
                let exclusion_regexes= to_vec_regex(&vec!(String::from("^https://www.pinterest.com/pin/create")));
                let links = extract_links(&lines, &regexes, &exclusion_regexes);
                println!("{:?}", links);
            }

            results.push(SearchResult {
                path: path,
                href: rel_path,
                lines
            });

        }
    }
}

fn to_vec_regex(vec: &Vec<String>) -> Vec<Regex> {
    vec.iter()
        .map(|string: &String| Regex::new(string.as_str()))
        .filter(|result| result.is_ok())
        .map(|result| result.unwrap())
        .collect()
}

fn extract_links(lines: &Vec<String>, regexes: &Vec<Regex>, exclusion_regexes: &Vec<Regex>) -> Vec<String> {
    let regex_href = Regex::new(r#"href="([^"]+)"#).unwrap();
    lines.iter()
        .map(|line| regex_href.captures(line))
        .filter(|capture_option| capture_option.is_some())
        .map(|capture_option| capture_option.unwrap())
        .map(|capture| capture.get(1) )
        .filter(|match_option| match_option.is_some())
        .map(|match_option| match_option.unwrap())
        .filter(|match_item| !match_item.is_empty())
        .map(|match_item| match_item.as_str().to_string())
        .filter(|link| match_one_regex(regexes, link) && match_no_regex(exclusion_regexes, link))
        .collect()
}

fn match_no_regex(regexes: &Vec<Regex>, subject: &String) -> bool {
    regexes.iter().find(|regex| regex.is_match(subject)).is_none()
}

fn match_one_regex(regexes: &Vec<Regex>, subject: &String) -> bool {
    regexes.iter().find(|regex| regex.is_match(subject)).is_some()
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
