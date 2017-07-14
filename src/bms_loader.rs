extern crate music;
use rand::{self, Rng};
use bms_parser::{BmsParser, BmsFileParser, BmsScript};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct KeyMetadata {
    id: u32,
    channel: String,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct MusicX {
    pub id: u32,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Debug)]
pub struct SoundX {
    pub id: u32,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Debug)]
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

#[derive(PartialEq, PartialOrd)]
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

#[derive(Debug)]
struct BmsEvent {
    segment_position: f64,
    event: BmsEventType
}

impl BmsEvent {
    fn new(segment_position: f64, event: BmsEventType) -> BmsEvent {
        BmsEvent { segment_position: segment_position, event: event }
    }
}

#[derive(Debug)]
enum BmsEventType {
    Bar,
    BpmChange(f64),
    Key(Key, SoundX),
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

    fn decompose_command(commands: &str) -> Vec<&str> {
        let notes = commands.len() / 2;
        let mut command_v = vec![];

        for idx in 0..notes {
            let fr = 2 * idx;
            let to = fr + 2;
            command_v.push(&commands[fr..to]);
        }

        command_v
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
        let mut wav_ids: HashSet<u32> = HashSet::new();
        for (key, value) in script.headers() {
            if key.starts_with("WAV") {
                let wav_id = u32::from_str_radix(&key[3..5], 36).unwrap();
                println!("{} {}", &value, path_path.with_file_name(&value).with_extension("ogg").as_path().to_str().unwrap());
                music::bind_sound_file(SoundX {id: wav_id}, path_path.with_file_name(&value).with_extension("ogg").as_path().to_str().unwrap());
                wav_ids.insert(wav_id);
            }
        }

        // parse from beginning
        let mut current_bpm = initial_bpm;
        let mut segment_head: f64 = 0.;
        for segment_id in &segment_ids {
            let mut events: Vec<BmsEvent> = vec![];
            events.push(BmsEvent::new(0., BmsEventType::Bar));

            let size_key = format!("{}{}", segment_id, "02");

            let segment_size: f64 = match script.channels().get(&size_key) {
                Some(s) => s.last().unwrap().trim().parse().unwrap(),
                None => 1.,
            };
            let beats: f64 = 4. * segment_size;

            let empty = vec![];
            // parse bpm change
            // TODO: handle channel 08 soft landing
            let softlanding_channel = format!("{}03", segment_id);
            for softlanding_channel_commands in script.channels().get(&softlanding_channel).unwrap_or(&empty) {
                let commands = BmsFileLoader::decompose_command(softlanding_channel_commands);
                let notes = commands.len();
                for (idx, command) in commands.iter().enumerate() {
                    let segment_position = (idx as f64) / (notes as f64);
                    let new_bpm = BmsFileLoader::decode(command);
                    if new_bpm > 0 {
                        events.push(BmsEvent::new(segment_position, BmsEventType::BpmChange(new_bpm as f64)))
                    }
                }
            }

            // parse keys
            for key in &keys {
                let channel_key = format!("{}{}", segment_id, channel_of_key(key));
                for channel_commands in script.channels().get(&channel_key).unwrap_or(&empty) {
                    let commands = BmsFileLoader::decompose_command(channel_commands);
                    let notes = commands.len();

                    for (idx, command) in commands.iter().enumerate() {
                        let wav_id = u32::from_str_radix(command, 36).unwrap();
                        let segment_position = (idx as f64) / (notes as f64);

                        if wav_id != 0 && wav_ids.contains(&wav_id) {
                            events.push(BmsEvent::new(segment_position, BmsEventType::Key(*key, SoundX {id: wav_id})));
                        };
                    };
                };
            };

            events.sort_by(|a, b| a.segment_position.partial_cmp(&b.segment_position).unwrap());
            let mut previous_position: f64 = 0.;
            let mut previous_timing: f64 = segment_head;
            let mut current_segment_bpm: f64 = current_bpm;
            for event in events {
                let position_delta = event.segment_position - previous_position;
                let timing_delta = position_delta * beats * BmsFileLoader::beat_duration(current_segment_bpm);
                let timing = previous_timing + timing_delta;

                println!("{} {:?}", timing, event);

                match event.event {
                    BmsEventType::Bar => bars.push(timing),
                    BmsEventType::Key(key, soundx) => sounds.push(Sound { key: key, timing: timing, wav_id: soundx} ),
                    BmsEventType::BpmChange(newBpm) => {
                        current_segment_bpm = newBpm;
                        bpms.push(BpmChange { timing: timing, bpm: newBpm} );
                    },
                };

                previous_position = event.segment_position;
                previous_timing = timing;
            }

            current_bpm = current_segment_bpm;
            segment_head = previous_timing + (1. - previous_position) * beats * BmsFileLoader::beat_duration(current_bpm);
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
