use std::fs::File;
use std::io::prelude::*;
use regex::Regex;
use std::collections::HashMap;


pub struct BmsScript {
    headers: HashMap<String, String>
}

impl BmsScript {
    fn wav_file(&self, key: &str) -> &str {
        unimplemented!()
    }
}

pub trait BmsParser {
    fn parse(&self) -> BmsScript;
}

pub struct BmsFileParser {
    path: String
}

impl BmsParser for BmsFileParser {
    fn parse(&self) -> BmsScript {
        let mut file = File::open(&self.path).expect("failed to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("failed to read file");
        BmsStringParser { script: contents}.parse()
    }
}

struct BmsStringParser {
    script: String
}

impl BmsParser for BmsStringParser {
    fn parse(&self) -> BmsScript {
        let mut headers = HashMap::new();
        for line in self.script.split('\n') {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue // empty line must be ignored
            } else {
                match &trimmed[..1] {
                    "#" => {
                        // command line or channel line
                        let channel_re = Regex::new(r"^#\d{5}:.*$").unwrap();
                        if channel_re.is_match(&trimmed) {
                            println!("channel: {}", line);
                        } else {
                            let tokens: Vec<&str> = trimmed.split(' ').collect();
                            let key = &tokens.get(0).unwrap()[1..];
                            let value = tokens[1..].join(" ");
                            headers.insert(key.to_string(), value.to_string());
                            println!("command: {} = {}", key, value);
                        }
                    },
                    _ => continue // comment line
                };
            };
        };
        BmsScript { headers: headers }
    }
}

#[test]
fn parser_test() {
    let bms = BmsFileParser { path: "example/conflict/_01_conflict.bme".to_string() }.parse();
    println!("{}", bms.headers.get("GENRE").unwrap())
}