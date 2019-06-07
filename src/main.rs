#[macro_use]
extern crate lazy_static;

use std::fs::{self, DirEntry};
use std::io::{self, Read, Write};
use std::path::Path;

const PUB: &[char] = &['p', 'u', 'b'];
lazy_static! {
    static ref LAB_PATH: std::path::PathBuf = std::env::temp_dir().join("bpr");
}
// one possible implementation of walking a directory only visiting files
fn visit_dirs(
    dir: &Path,
    cb: &Fn(&DirEntry, &mut Vec<(std::path::PathBuf, usize, usize)>) -> io::Result<()>,
    map: &mut Vec<(std::path::PathBuf, usize, usize)>,
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

fn check_pub(file: &DirEntry, map: &mut Vec<(std::path::PathBuf, usize, usize)>) -> io::Result<()> {
    let mut f = std::fs::File::open(file.path())?;
    let mut b = String::new();
    f.read_to_string(&mut b)?;
    let b: Vec<char> = b.chars().collect();
    let mut cummulative_line_len = vec![];
    for (idx, line) in b.split(|c| *c == '\n').enumerate() {
        cummulative_line_len.push(line.len() + *cummulative_line_len.last().unwrap_or(&0) + 1);
        // ignore comments
        {
            let line: String = line.iter().collect();
            if line.trim_start().starts_with("//") {
                continue;
            }
        }

        let mut c = 0;
        loop {
            match pub_found(&line, c) {
                Some(true) => {
                    if pub_is_needless(
                        &mut b.clone(),
                        c + cummulative_line_len
                            [cummulative_line_len.len().checked_sub(2).unwrap_or(0)],
                        file,
                    ) {
                        map.push((file.path(), idx + 1, c));
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
    //dbg!(&out);

    let is_needless = !out.contains("E0624") && !out.contains("E0603") && !out.contains("E0616");

    if is_needless && std::env::args().nth(1) == Some("-i".into()) {
        let origin_path = std::path::Path::new("./").join(file.path().strip_prefix(LAB_PATH.as_path()).unwrap());
        let mut f = std::fs::File::create(origin_path).unwrap();
        write!(f, "{}", b.iter().collect::<String>()).unwrap();
    }

    // reinsert pub keyword
    for letter in PUB.iter().rev() {
        b.insert(file_idx, *letter);
    }
    //let mut f = std::fs::File::create(file.path()).unwrap();
    //write!(f, "{}", b.iter().collect::<String>()).unwrap();



    is_needless
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
        if src == Path::new("./target") {
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

fn main() {
    let _ = std::fs::remove_dir_all(LAB_PATH.as_path());
    copy_entry(Path::new("./"), &LAB_PATH).unwrap();
    let mut indexes = Vec::new();
    visit_dirs(&LAB_PATH, &check_pub, &mut indexes).unwrap();
    dbg!(indexes);
}
