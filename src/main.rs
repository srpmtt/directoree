use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

fn read_gitignore<P: AsRef<Path>>(path: P) -> Vec<String> {
    let gitignore_path = path.as_ref().join(".gitignore");
    let mut ignore_entries = Vec::new();

    if let Ok(file) = fs::File::open(gitignore_path) {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    ignore_entries.push(trimmed.to_string());
                }
            }
        }
    }

    ignore_entries
}

fn matches_gitignore(entry: &str, ignored_entries: &[String]) -> bool {
    for ignored_entry in ignored_entries {
        if entry == ignored_entry || entry.starts_with(ignored_entry.trim_end_matches('/')) {
            return true;
        }
    }
    false
}

fn print_tree<P: AsRef<Path>>(
    path: P,
    prefix: &str,
    show_all: bool,
    gitignore_patterns: &[String],
) {
    let path = path.as_ref();

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("Error reading directory {}: {}", path.display(), err);
            return;
        }
    };

    let entries_vec: Vec<(PathBuf, bool)> = entries
        .filter_map(Result::ok)
        .map(|entry| (entry.path(), entry.path().is_dir()))
        .filter(|(entry_path, _)| {
            let entry_name = entry_path.file_name().unwrap().to_string_lossy();
            let is_hidden = entry_name.starts_with('.');
            let is_ignored = matches_gitignore(&entry_name, gitignore_patterns);
            show_all || (!is_hidden && !is_ignored)
        })
        .collect();

    let total_entries = entries_vec.len();

    for (i, (entry_path, is_dir)) in entries_vec.iter().enumerate() {
        let is_last = i == total_entries - 1;
        let entry_name = entry_path.file_name().unwrap().to_string_lossy();

        println!(
            "{}{} {}",
            prefix,
            if is_last { "└── " } else { "├── " },
            entry_name
        );

        if *is_dir {
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│  " });
            print_tree(entry_path, &new_prefix, show_all, gitignore_patterns);
        }
    }
}

fn main() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let gitignore_patterns = read_gitignore(&current_dir);
    let show_all = std::env::args().any(|arg| arg == "-a" || arg == "--all");
    println!("{}", current_dir.display());
    print_tree(current_dir, "", show_all, &gitignore_patterns);
}
