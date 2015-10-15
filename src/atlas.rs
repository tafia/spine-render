use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use std::io::Lines;
use std::io::prelude::*;

pub struct Texture {
    pub rotate: bool,
    pub xy: (u16, u16),
    pub size: (u16, u16),
    pub orig: (u16, u16),
    pub offset: (u16, u16),
    pub index: i16,
}

pub struct Atlas {
    pub textures: HashMap<String, Texture>
}

impl Atlas {

    pub fn load(src: &str) -> Atlas {

        let f = File::open(src).unwrap();
        let reader = BufReader::new(f);
        let mut lines = reader.lines();

        let mut textures = HashMap::new();
        loop {
            if let Some((name, line)) = next_texture(&mut lines) {
                let txt = read_texture(&line, &mut lines);
                textures.insert(name, txt);
            } else {
                return Atlas {
                    textures: textures,
                };
            }
        }
    }
}

fn next_texture(lines: &mut Lines<BufReader<File>>) -> Option<(String, String)> {
    let mut old_line = String::new();
    loop {
        if let Some(Ok(line)) = lines.next() {
            if line.starts_with("\t") || line.starts_with(" ") {
                return Some((old_line, line));
            }
            old_line = line;
        } else {
            return None;
        }
    }
}

fn read_texture(line: &str, lines: &mut Lines<BufReader<File>>) -> Texture  {
    let rotate = line.trim()["rotate:".len()..].trim().parse().unwrap();
    let mut line = lines.next().unwrap().unwrap();
    let xy = parse_tuple(&line.trim()["xy:".len()..]);
    line = lines.next().unwrap().unwrap();
    let size = parse_tuple(&line.trim()["size:".len()..]);
    line = lines.next().unwrap().unwrap();
    let orig = parse_tuple(&line.trim()["orig:".len()..]);
    line = lines.next().unwrap().unwrap();
    let offset = parse_tuple(&line.trim()["offset:".len()..]);
    line = lines.next().unwrap().unwrap();
    let index = line.trim()["index:".len()..].trim().parse().unwrap();
    Texture {
        rotate: rotate,
        xy: xy,
        size: size,
        orig: orig,
        offset: offset,
        index: index,
    }
}

fn parse_tuple(s: &str) -> (u16, u16) {
    let mut splits = s.split(',');
    (splits.next().unwrap().trim().parse().unwrap(),
     splits.next().unwrap().trim().parse().unwrap())
}
