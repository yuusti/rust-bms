use std::fs::File;
use std::io::prelude::*;


pub struct BmsScript {

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
        println!("loaded: {}", contents);
        BmsStringParser { script: contents}.parse()
    }
}

struct BmsStringParser {
    script: String
}

impl BmsParser for BmsStringParser {
    fn parse(&self) -> BmsScript {
        println!("start parsing");
        for line in self.script.split('\n') {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                println!("empty line");
            } else {
                match &trimmed[..1] {
                    "#" => {
                        // command line or channel line
                        println!("command or channel: {}", line);
                    },
                    _ => {
                        // comment line
                        println!("comment line: {}", line);
                    }
                };
            };
        };
        BmsScript {}
    }
}

#[test]
fn parser_test() {
    println!("rust");

    BmsFileParser { path: "example/conflict/_01_conflict.bme".to_string() }.parse();
}