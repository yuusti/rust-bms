use std::fs::File;
use std::io::prelude::*;
use regex::Regex;
use std::collections::HashMap;


pub struct BmsScript {
    headers: HashMap<String, String>,
    channels: HashMap<String, String>,
}

impl BmsScript {
    fn wav_file(&self, key: &str) -> &str {
        unimplemented!()
    }

    fn channel(&self, segment_id: u32, channel_id: u32) -> &str {
        let key = format!("{0: <03}{1: <02}", segment_id, channel_id);
        println!("{}", key);
        self.channels.get(&key).unwrap()
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
        let mut channels = HashMap::new();
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
                            let key = &line[1..6];
                            let value = &line[8..];
                            channels.insert(key.to_string(), value.to_string());
                            println!("channel: {} = {}", key, value);
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
        BmsScript { headers: headers, channels: channels }
    }
}

#[test]
fn parser_test() {
    let bms = BmsFileParser { path: "example/conflict/_01_conflict.bme".to_string() }.parse();
    println!("{}", bms.headers.get("GENRE").unwrap());
    println!("{}", bms.channel(1, 1))
}