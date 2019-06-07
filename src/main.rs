use std::fs::{self, DirEntry};
pub use std::io::{self, Read, Write};
use std::path::Path;

const PUB: &[char] = &['p', 'u', 'b'];

// one possible implementation of walking a directory only visiting files
fn visit_dirs(
    dir: &Path,
    cb: &Fn(&DirEntry, &mut Vec<(usize, usize)>) -> io::Result<()>,
    map: &mut Vec<(usize, usize)>,
) -> io::Result<()> {
    //dbg!(&dir);
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            //      dbg!(4);
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb, map)?;
            } else {
                cb(&entry, map).unwrap();
            }
        }
    }
    Ok(())
}

fn check_pub(file: &DirEntry, map: &mut Vec<(usize, usize)>) -> io::Result<()> {
    let mut f = std::fs::File::open(file.path())?;
    let mut b = String::new();
    f.read_to_string(&mut b)?;
    for (idx, line) in b.lines().enumerate() {
        let mut b:Vec<char> = line.chars().collect();
        let mut c = 0;
        loop {
            match pub_found(&b, c) {
                Some(true) => {
                    if pub_is_needless(&mut b, c, file) {
                        map.push((idx, c));
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

fn pub_is_needless(b: &mut Vec<char>, c: usize, file: &DirEntry) -> bool {
    // remove pub keyword
    //    dbg!(c);
    //  dbg!(&b[..10]);
    for _ in 0..3 {
        b.remove(c);
    }

    //dbg!(&b[..10]);

    let mut f = std::fs::File::create(file.path()).unwrap();
    write!(f, "{}", b.iter().collect::<String>()).unwrap();
    let out = std::process::Command::new("cargo")
        .arg("b")
        .current_dir("./")
        .output()
        .unwrap();

    let out = if out.stdout.is_empty() {
        out.stderr
    } else {
        out.stdout
    };
    let out = String::from_utf8(out);
    dbg!(&out);

    // reinsert pub keyword
    for i in 0..3 {
        b.insert(c, PUB[i]);
    }
    let mut f = std::fs::File::create(file.path()).unwrap();
    write!(f, "{}", b.iter().collect::<String>()).unwrap();
    //dbg!(&b[..10]);
    //loop {}
    true
}

fn pub_found(v: &[char], c: usize) -> Option<bool> {
    let found = concat(&[v.get(c)?, v.get(c + 1)?, v.get(c + 2)?]) == PUB;
    let mut found = found && !(*v.get(c + 3)? as char).is_alphanumeric();
    if c > 0 {
        found = found && !(*v.get(c - 1)? as char).is_alphanumeric();
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

fn main() {
    let mut indexes = Vec::new();
    visit_dirs(std::path::Path::new("./src"), &check_pub, &mut indexes).unwrap();
    dbg!(indexes);
}
