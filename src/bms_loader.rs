extern crate music;
use rand::{self, Rng};
use bms_parser::{BmsParser, BmsFileParser, BmsScript};
use std::collections::HashSet;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct KeyMetadata {
    id: u32,
    channel: String,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct MusicX {
    pub id: u32,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct SoundX {
    pub id: u32,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum Key {
    P1_KEY1 = 1,
    P1_KEY2 = 2,
    P1_KEY3 = 3,
    P1_KEY4 = 4,
    P1_KEY5 = 5,
    P1_KEY6 = 6,
    P1_KEY7 = 7,
    P1_SCRATCH = 0,
    P1_FREE_SCRATCH = 254,
    BACK_CHORUS = 255,
}

impl Key {
    pub fn visible_keys() -> HashSet<Key> {
        vec![Key::P1_KEY1,
             Key::P1_KEY2,
             Key::P1_KEY3,
             Key::P1_KEY4,
             Key::P1_KEY5,
             Key::P1_KEY6,
             Key::P1_KEY7,
             Key::P1_SCRATCH].into_iter().collect()
    }
}

fn channel_of_key(key: &Key) -> &'static str {
    match *key {
        Key::P1_KEY1 => "11",
        Key::P1_KEY2 => "12",
        Key::P1_KEY3 => "13",
        Key::P1_KEY4 => "14",
        Key::P1_KEY5 => "15",
        Key::P1_KEY6 => "18",
        Key::P1_KEY7 => "19",
        Key::P1_SCRATCH => "16",
        Key::BACK_CHORUS => "01",
        _ => "none",
    }
}

pub struct Sound {
    pub key: Key,
    pub timing: f64,
    pub wav_id: SoundX,
}

// pub struct Sound<'a> {
//    pub key: Key,
//    pub timing: f64,
//    pub handle: &'a ears::Sound,
// }

pub struct BpmChange {
    pub timing: f64,
    pub bpm: f64,
}

pub struct Bms {
    pub sounds: Vec<Sound>,
    pub bars: Vec<f64>,  // time for bar line to pass the judge line relative to start time in sec.
    pub bpms: Vec<BpmChange>,
}

pub trait BmsLoader {
    fn load(&self) -> Bms;
}

pub struct BmsFileLoader {
    path: String
}

impl BmsFileLoader {
    pub fn new(path: &str) -> BmsFileLoader {
        BmsFileLoader { path: path.to_string() }
    }

    fn list_segment_ids(script: &BmsScript) -> Vec<&str> {
        let channels = script.channels().keys();
        let mut ret: Vec<&str> = vec![];
        for key in channels {
            ret.push(&key[..3])
        }
        ret.sort();
        ret.dedup();
        ret
    }

    fn decode(code: &str) -> i32 {
        i32::from_str_radix(code, 16).unwrap()
    }

    fn beat_duration(bpm: f64) -> f64 {
        60. / bpm
    }
}

impl BmsLoader for BmsFileLoader {
    fn load(&self) -> Bms {
        let script_parser = BmsFileParser { path: self.path.to_string() };
        let script = script_parser.parse();

        let segment_ids = BmsFileLoader::list_segment_ids(&script);

        // 1. make bpms
        let mut bpms: Vec<BpmChange> = vec![];
        let mut bars: Vec<f64> = vec![];
        let mut sounds: Vec<Sound> = vec![];

        // get initial bpm
        let initial_bpm: f64 = script.headers().get("BPM").unwrap().parse().unwrap_or(130.);

        bpms.push(BpmChange { timing: 0., bpm: initial_bpm });

        // TODO: DP
        let keys = vec![
            Key::BACK_CHORUS,
            Key::P1_KEY1,
            Key::P1_KEY2,
            Key::P1_KEY3,
            Key::P1_KEY4,
            Key::P1_KEY5,
            Key::P1_KEY6,
            Key::P1_KEY7,
            Key::P1_SCRATCH,
        ];

        let path_path = Path::new(&self.path);
        for (key, value) in script.headers() {
            if key.starts_with("WAV") {
                //let filename = path_path.with_file_name(&value).with_extension("ogg").as_path().to_str().unwrap();
                println!("{} {}", &value, path_path.with_file_name(&value).with_extension("ogg").as_path().to_str().unwrap());
                music::bind_sound_file(SoundX {id: u32::from_str_radix(&key[3..5], 36).unwrap()}, path_path.with_file_name(&value).with_extension("ogg").as_path().to_str().unwrap());
            }
        }

        // parse from beginning
        let mut current_bpm = initial_bpm;
        let mut segment_head: f64 = 0.;
        for segment_id in &segment_ids {
            bars.push(segment_head);

            let size_key = format!("{}{}", segment_id, "02");

            let segment_size: f64 = match script.channels().get(&size_key) {
                Some(s) => s.trim().parse().unwrap(),
                None => 1.,
            };
            let beats: f64 = 4. * segment_size;
            // TODO: handle soft landing
            let segment_duration = BmsFileLoader::beat_duration(current_bpm) * beats;

            for key in &keys {
                let channel_key = format!("{}{}", segment_id, channel_of_key(key));
                let empty = "00".to_string();
                let channel_commands = script.channels().get(&channel_key).unwrap_or(&empty);

                let notes = channel_commands.len() / 2;
                // TODO: handle soft landing
                let notes_interval = segment_duration / (notes as f64);

                for idx in 0..notes {
                    let wav_id = &channel_commands[2*idx..(2*idx + 2)];
                    let timing = segment_head + (idx as f64) * notes_interval;

                    if wav_id != "00" {
                        println!("{} {}", wav_id, timing);
                        sounds.push(Sound {key: *key, timing: timing, wav_id: SoundX {id: u32::from_str_radix(wav_id, 36).unwrap()}});
                    }
                };
            };

            segment_head += segment_duration;
        };

        println!("notes: {}", sounds.len());

        Bms { bpms: bpms, bars: bars, sounds: sounds }
    }
}

pub struct FixtureLoader { i: i32 }

impl FixtureLoader {
    pub fn new() -> FixtureLoader {
        FixtureLoader { i: 0 }
    }
}

impl BmsLoader for FixtureLoader {
    fn load(&self) -> Bms {
        let keys = vec![
            Key::P1_KEY1,
            Key::P1_KEY2,
            Key::P1_KEY3,
            Key::P1_KEY4,
            Key::P1_KEY5,
            Key::P1_KEY6,
            Key::P1_KEY7,
            Key::P1_SCRATCH,
        ];

        let mut rng = rand::thread_rng();

        let mut v = vec![];
        //wavs.insert("01".to_owned(), None);
        for i in 0..10000 {
            v.push(
                Sound { key: keys[i % keys.len()], timing: rng.gen_range(1f64, 1000f64), wav_id: SoundX { id: 1 } },
            )
        }
        v.sort_by(|a, b| a.timing.partial_cmp(&b.timing).unwrap());

        use std::f64;
        Bms {
            sounds: v,
            bars: (0..1000i64).map(|x| x as f64).collect(),
            bpms: (0..100000i64).map(|x| BpmChange { timing: x as f64 / 100.0, bpm: 201.0 + 200.0 * ((x as f64 / 100.0 % (f64::consts::PI * 2.0)).sin()) }).collect()
        }
    }
}

#[test]
fn loader_test() {
    let loader = BmsFileLoader { path: "example/conflict/_01_conflict.bme".to_string() };
    let bms = loader.load();
}
