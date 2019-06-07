#[macro_use]
extern crate lazy_static;

use std::fs::{self, DirEntry};
use std::io::{self, Read, Write};
use std::path::Path;

type Map = Vec<(std::path::PathBuf, usize, usize, usize)>;
type ArrMap = [(std::path::PathBuf, usize, usize, usize)];

const PUB: &[char] = &['p', 'u', 'b'];
lazy_static! {
    static ref LAB_PATH: std::path::PathBuf = std::env::temp_dir().join("bpr");
    static ref PROJECT_PATH: std::path::PathBuf = {
        let custom_path = std::env::args().skip(1).filter(|a| a != "-i").last();
        if let Some(p) = custom_path {
            p.into()
        } else {
            "./".into()
        }
    };
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(
    dir: &Path,
    cb: &Fn(&DirEntry, &mut Map) -> io::Result<()>,
    map: &mut Map,
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb, map)?;
            } else {
                let _ = cb(&entry, map);
            }
        }
    }
    Ok(())
}

fn check_pub(file: &DirEntry, map: &mut Map) -> io::Result<()> {
    if !is_rust_file(file.path()).unwrap_or(false) {
        return Ok(());
    }

    let mut f = std::fs::File::open(file.path())?;
    let mut b = String::new();
    f.read_to_string(&mut b)?;
    let b: Vec<char> = b.chars().collect();
    let mut cursor = vec![0];
    for (idx, line) in b.split(|c| *c == '\n').enumerate() {
        cursor.push(line.len() + *cursor.last().unwrap_or(&0) + 1);
        // ignore comments
        {
            let line: String = line.iter().collect();
            if line.trim_start().starts_with("//") {
                continue;
            }
        }

        let mut c = 0;
        loop {
            let cursor = c + cursor[cursor.len().checked_sub(2).unwrap_or(0)];
            match pub_found(&line, c) {
                Some(true) => {
                    if pub_is_needless(&mut b.clone(), cursor, file) {
                        map.push((tmp_to_origin(file.path()), idx + 1, c, cursor));
                    }
                }
                Some(_) => {}
                None => break,
            };
            c += 1;
        }
    }

    Ok(())
}

fn pub_is_needless(b: &mut Vec<char>, file_idx: usize, file: &DirEntry) -> bool {
    // remove pub keyword
    for _ in 0..3 {
        b.remove(file_idx);
    }

    let mut f = std::fs::File::create(file.path()).unwrap();
    write!(f, "{}", b.iter().collect::<String>()).unwrap();
    //loop {}

    let out = std::process::Command::new("cargo")
        .arg("b")
        .current_dir(LAB_PATH.as_path())
        .output()
        .unwrap();

    let out = if out.stdout.is_empty() {
        out.stderr
    } else {
        out.stdout
    };
    let out = String::from_utf8(out).unwrap();

    // reinsert pub keyword
    for letter in PUB.iter().rev() {
        b.insert(file_idx, *letter);
    }

    // restore file
    let mut f = std::fs::File::create(file.path()).unwrap();
    write!(f, "{}", b.iter().collect::<String>()).unwrap();

    !out.contains("E0624")
        && !out.contains("E0603")
        && !out.contains("E0616")
        && !out.contains("E0433")
}

fn pub_found(v: &[char], c: usize) -> Option<bool> {
    let found = concat(&[v.get(c)?, v.get(c + 1)?, v.get(c + 2)?]) == PUB;
    let mut found = found && (*v.get(c + 3)? as char).is_whitespace();
    if c > 0 {
        found = found && (*v.get(c - 1)? as char).is_whitespace();
    }
    Some(found)
}

fn concat<T: Copy>(l: &[&T]) -> Vec<T> {
    let mut result = Vec::with_capacity(l.len());
    for v in l.iter() {
        result.push(**v);
    }
    result
}

fn copy_entry(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        if src == PROJECT_PATH.join("target") {
            return Ok(());
        }
        let _ = fs::DirBuilder::new().create(&dst);

        for sub_entry in fs::read_dir(src)? {
            let sub_entry = sub_entry?;
            let path = sub_entry.path();
            let dst = dst.join(&path.file_name().unwrap());
            copy_entry(&path, &dst)?;
        }
    } else {
        fs::copy(src, dst)?;
    }
    Ok(())
}

fn is_rust_file(f: std::path::PathBuf) -> Option<bool> {
    Some(f.iter().last()?.to_str()?.ends_with("rs"))
}

fn tmp_to_origin(f: std::path::PathBuf) -> std::path::PathBuf {
    PROJECT_PATH.join(f.strip_prefix(LAB_PATH.as_path()).unwrap())
}

fn apply_changes(v: &ArrMap) {
    let mut current_f = v.first().unwrap().0.clone();
    let mut idx = 0;

    for (f_path, _, _, cursor) in v.into_iter() {
        if &current_f != f_path {
            idx = 0;
            current_f = f_path.clone();
        }
        let fixed_cursor = cursor - 4 * idx;
        idx += 1;

        let mut b = String::new();
        let mut f = std::fs::File::open(&f_path).unwrap();

        f.read_to_string(&mut b).unwrap();
        let mut b: Vec<char> = b.chars().collect();

        for _ in 0..4 {
            b.remove(fixed_cursor);
        }
        let mut f = std::fs::File::create(&f_path).unwrap();
        write!(f, "{}", b.iter().collect::<String>()).unwrap();
    }
}

fn print_result(v: &ArrMap) {
    for (file, x, y, _) in v {
        println!(
            "needles pub found in file: {} at row: {} col: {}",
            file.display(),
            x,
            y
        );
    }
}

fn is_bin(p: &std::path::Path) -> std::io::Result<bool> {
    let out = std::process::Command::new("cargo")
        .arg("r")
        .current_dir(p)
        .output()?
        .stderr;
    Ok(String::from_utf8(out)
        .unwrap()
        .contains("Finished dev [unoptimized + debuginfo]"))
}

fn main() {
    if !is_bin(&PROJECT_PATH).unwrap() {
        eprintln!("{} is not a binary rust project", PROJECT_PATH.display());
        std::process::exit(1);
    };
    let _ = std::fs::remove_dir_all(LAB_PATH.as_path());

    copy_entry(&PROJECT_PATH, &LAB_PATH).unwrap();
    let mut indexes = Vec::new();
    visit_dirs(&LAB_PATH.join("src"), &check_pub, &mut indexes).unwrap();

    if std::env::args().nth(1) == Some("-i".into()) && !indexes.is_empty() {
        apply_changes(&indexes);
    }

    print_result(&indexes);
}
