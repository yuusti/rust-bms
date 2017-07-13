use ears;
use rand::{self, Rng};

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
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
    pub fn visible_keys() -> Vec<Key> {
        vec![Key::P1_KEY1,
             Key::P1_KEY2,
             Key::P1_KEY3,
             Key::P1_KEY4,
             Key::P1_KEY5,
             Key::P1_KEY6,
             Key::P1_KEY7,
             Key::P1_SCRATCH]
    }
}

pub struct Sound<'a> {
    pub key: Key,
    pub timing: f64,
    pub handle: &'a i32,
}

//pub struct Sound<'a> {
//    pub key: Key,
//    pub timing: f64,
//    pub handle: &'a ears::Sound,
//}

pub struct BpmChange {
    pub timing: f64,
    pub bpm: f64,
}

pub struct Bms<'a> {
    pub sounds: Vec<Sound<'a>>,
    pub bars: Vec<f64>,
    pub bpms: Vec<BpmChange>,
}

pub trait BmsLoader {
    fn load(&self) -> Bms;
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
                Sound { key: keys[i % keys.len()], timing: rng.gen_range(1f64, 1000f64), handle: &self.i },
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

