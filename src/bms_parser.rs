use std::fs::File;
use std::io::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use time;

pub struct BmsScript {
    headers: HashMap<String, String>,
    channels: HashMap<String, Vec<String>>,
}

impl BmsScript {
    pub fn wav_file(&self, key: &str) -> &str {
        unimplemented!()
    }

    pub fn channels(&self) -> &HashMap<String, Vec<String>> {
        &self.channels
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn channel(&self, segment_id: &str, channel_id: &str) -> &Vec<String> {
        let key = format!("{0}{1}", segment_id, channel_id);
        self.channels.get(&key).unwrap()
    }

    pub fn header(&self, key: &str) -> &str {
        self.headers.get(key).unwrap()
    }

}

pub trait BmsParser {
    fn parse(&self) -> BmsScript;
}

pub struct BmsFileParser {
    pub path: String
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
        println!("Start BmsStringParser::parse() at {}", time::precise_time_s());

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
                            let value = &line[7..];
                            if !channels.contains_key(key) {
                                channels.insert(key.to_string(), vec![]);
                            }
                            channels.get_mut(key).map(|x| x.push(value.to_string()));
                        } else {
                            let tokens: Vec<&str> = trimmed.split(' ').collect();
                            let key = &tokens.get(0).unwrap()[1..];
                            let value = tokens[1..].join(" ");
                            headers.insert(key.to_string(), value.to_string());
                        }
                    },
                    _ => continue // comment line
                };
            };
        };
        println!("Finish BmsStringParser::parse() at {}", time::precise_time_s());
        BmsScript { headers: headers, channels: channels }
    }
}

#[test]
fn parser_test() {
    let bms = BmsFileParser { path: "example/conflict/_01_conflict.bme".to_string() }.parse();
    println!("{}", bms.headers().get("GENRE").unwrap());
    println!("{}", bms.channel("091", "06"))
}