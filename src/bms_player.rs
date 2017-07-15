extern crate music;

use time;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use graphics::rectangle::square;

use std::path::Path;
use bms_loader::{self, Bms, Sound};
use std::collections::{HashSet, HashMap};
use ears;
use ears::{AudioController};

type Time = f64;

pub struct BmsPlayer {
    speed: f64,
    bpm: f64,
    obj_index_by_key: HashMap<bms_loader::Key, usize>,
    event_index: usize,
    objects_by_key: HashMap<bms_loader::Key, Vec<Draw>>,
    events: Vec<Event>,
    judge_index_by_key: HashMap<bms_loader::Key, usize>,
    pushed_key_set: HashSet<bms_loader::Key>,
    judge_display: JudgeDisplay,
    y_offset: f64,
    bpms: Vec<bms_loader::BpmChange>,
    init_time: Option<f64>,
    textures_map: HashMap<TextureLabel, Texture>,
    key_mapping: HashMap<Key, bms_loader::Key>,
}

#[inline]
fn note_info(key: bms_loader::Key) -> Option<(f64, f64, TextureLabel)> {
    // x pos, size, color
    use bms_loader::Key;
    let x = key as u8;

    match key {
        Key::P1_KEY1 | Key::P1_KEY3 | Key::P1_KEY5 | Key::P1_KEY7 => {
            Some((SCR_WIDTH + NOTES1_WIDTH * (x / 2) as f64 + NOTES2_WIDTH * ((x - 1) / 2) as f64 + OFFSET, NOTES1_WIDTH - OFFSET * 2.0, TextureLabel::NOTE_WHITE))
        }
        Key::P1_KEY2 | Key::P1_KEY4 | Key::P1_KEY6 => {
            Some((SCR_WIDTH + NOTES1_WIDTH * (x / 2) as f64 + NOTES2_WIDTH * ((x - 1) / 2) as f64 + OFFSET, NOTES2_WIDTH - OFFSET * 2.0, TextureLabel::NOTE_BLUE))
        }
        Key::P1_SCRATCH => {
            Some((OFFSET, SCR_WIDTH - OFFSET * 2.0, TextureLabel::NOTE_RED))
        }
        _ => None,
    }
}

#[inline]
fn beam_info(key: bms_loader::Key) -> Option<(f64, f64, TextureLabel)> {
    // x pos, size, color
    use bms_loader::Key;
    let x = key as u8;

    match key {
        Key::P1_KEY1 | Key::P1_KEY3 | Key::P1_KEY5 | Key::P1_KEY7 => {
            Some((SCR_WIDTH + NOTES1_WIDTH * (x / 2) as f64 + NOTES2_WIDTH * ((x - 1) / 2) as f64 + OFFSET, NOTES1_WIDTH - OFFSET * 2.0, TextureLabel::WHITE_BEAM))
        }
        Key::P1_KEY2 | Key::P1_KEY4 | Key::P1_KEY6 => {
            Some((SCR_WIDTH + NOTES1_WIDTH * (x / 2) as f64 + NOTES2_WIDTH * ((x - 1) / 2) as f64 + OFFSET, NOTES2_WIDTH - OFFSET * 2.0, TextureLabel::BLUE_BEAM))
        }
        Key::P1_SCRATCH => {
            Some((OFFSET, SCR_WIDTH - OFFSET * 2.0, TextureLabel::RED_BEAM))
        }
        _ => None,
    }
}

const SCR_WIDTH: f64 = 108f64;
const NOTES1_WIDTH: f64 = 60f64;
const NOTES2_WIDTH: f64 = 50f64;
const NOTES_HEIGHT: f64 = 10.0;
const BAR_HEIGHT: f64 = 1.0;
const OFFSET: f64 = 2.5;

fn calc_position(t: Time, bpms: &Vec<bms_loader::BpmChange>) -> f64 {
    let mut y = 0f64;
    let mut p_bpm = 130f64;
    let mut p_timing = 0f64;
    for bpm in bpms {
        let d = if bpm.timing < t {
            bpm.timing - p_timing
        } else if t - p_timing > 0f64 {
            t - p_timing
        } else {
            0f64
        };
        if d < 0f64 {
            break;
        }
        y += d * p_bpm;
        p_bpm = bpm.bpm;
        p_timing = bpm.timing;
    }
    y += if t - p_timing > 0f64 {
        t - p_timing
    } else {
        0f64
    } * p_bpm;

    y
}

pub fn f64_eq(a: f64, b: f64) -> bool {
    f64::abs(a-b) < 1e-9
}

#[test]
pub fn test_calc_position () {
    let bpms = vec![
        bms_loader::BpmChange{timing: 0f64,bpm: 100f64},
        bms_loader::BpmChange{timing: 10f64,bpm: 200f64},
        bms_loader::BpmChange{timing: 20f64,bpm: 400f64},
    ];

    assert!(f64_eq(0f64, calc_position(0f64, &bpms)));
    assert!(f64_eq(500f64, calc_position(5f64, &bpms)));
    assert!(f64_eq(1000f64, calc_position(10f64, &bpms)));
    assert!(f64_eq(2000f64, calc_position(15f64, &bpms)));
    assert!(f64_eq(3000f64, calc_position(20f64, &bpms)));
    assert!(f64_eq(5000f64, calc_position(25f64, &bpms)));
    assert!(f64_eq(7000f64, calc_position(30f64, &bpms)));
}

impl BmsPlayer {
    pub fn new(
        textures_map: HashMap<TextureLabel, Texture>,
        bms: Bms,
        time: Time,
        speed: f64,
    ) -> BmsPlayer {
        println!("Start BmsPlayer Initialization at {}", time::precise_time_s());
        let mut objects_by_key = HashMap::new();
        for key in bms_loader::Key::visible_keys() {
            objects_by_key.insert(key, vec![]);
        }

        let mut events = vec![];
        for sound in bms.sounds {
            if bms_loader::Key::visible_keys().contains(&sound.key) {
                if let Some((x, width, texture_label)) = note_info(sound.key) {
                    objects_by_key.get_mut(&sound.key).unwrap().push(Draw { timing: sound.timing, x: x, y: calc_position(sound.timing, &bms.bpms) , width: width, height: NOTES_HEIGHT, texture_label: texture_label, wav_id: Some(sound.wav_id) });
                }
            } else if sound.key == bms_loader::Key::BACK_CHORUS {
                events.push(Event { timing: sound.timing, event_type: EventType::PlaySound(sound) });
            }
        }

        objects_by_key.insert(bms_loader::Key::BACK_CHORUS, vec![]);
        for bar in bms.bars.iter() {
            objects_by_key.get_mut(&bms_loader::Key::BACK_CHORUS).unwrap().push(Draw { timing: *bar, x: 0.0, y: calc_position(*bar, &bms.bpms), width: 1000.0, height: BAR_HEIGHT, texture_label: TextureLabel::BACKGROUND, wav_id: None });
        }

        let mut obj_index_by_key = HashMap::new();
        for (key, ref mut objects) in &mut objects_by_key {
            objects.sort_by(|a, b| a.timing.partial_cmp(&b.timing).unwrap());
            obj_index_by_key.insert(*key, 0usize);
        }

        for bpm in bms.bpms.iter() {
            events.push(Event { timing: bpm.timing, event_type: EventType::ChangeBpm(bpm.bpm) });
        }
        events.sort_by(|a, b| a.timing.partial_cmp(&b.timing).unwrap());

        let mut key_mapping = HashMap::new();
        key_mapping.insert(Key::A, bms_loader::Key::P1_SCRATCH);
        key_mapping.insert(Key::Z, bms_loader::Key::P1_KEY1);
        key_mapping.insert(Key::J, bms_loader::Key::P1_KEY1);
        key_mapping.insert(Key::S, bms_loader::Key::P1_KEY2);
        key_mapping.insert(Key::X, bms_loader::Key::P1_KEY3);
        key_mapping.insert(Key::K, bms_loader::Key::P1_KEY3);
        key_mapping.insert(Key::D, bms_loader::Key::P1_KEY4);
        key_mapping.insert(Key::C, bms_loader::Key::P1_KEY5);
        key_mapping.insert(Key::L, bms_loader::Key::P1_KEY5);
        key_mapping.insert(Key::F, bms_loader::Key::P1_KEY6);
        key_mapping.insert(Key::V, bms_loader::Key::P1_KEY7);
        key_mapping.insert(Key::Semicolon, bms_loader::Key::P1_KEY7);

        println!("Finish BmsPlayer Initialization at {}", time::precise_time_s());
        BmsPlayer {
            speed: speed,
            bpm: 130f64,
            obj_index_by_key: obj_index_by_key.clone(),
            event_index: 0usize,
            objects_by_key: objects_by_key,
            events: events,
            judge_index_by_key: obj_index_by_key.clone(),
            pushed_key_set: HashSet::new(),
            judge_display: JudgeDisplay::new(),
            y_offset: 0f64,
            bpms: bms.bpms,
            init_time: None,
            textures_map: textures_map,
            key_mapping: key_mapping,
        }
    }

    pub fn run(&mut self, window: &mut Window, gl :&mut GlGraphics) {
        let mut events = Events::new(EventSettings::new());

        music::set_volume(music::MAX_VOLUME);
        while let Some(e) = events.next(window) {
            if let Some(u) = e.update_args() {
                self.update(&u);
            }

            if let Some(r) = e.render_args() {
                self.render(&r, gl);
            }

            if let Some(Button::Keyboard(key)) = e.press_args() {
                self.on_key_down(&key);
            }

            if let Some(Button::Keyboard(key)) = e.release_args() {
                self.on_key_up(&key);
            }

            self.process_event();
        }
    }

    fn render(&mut self, args: &RenderArgs, gl :&mut GlGraphics) {
        let pt = self.get_precise_time();
        self.y_offset = calc_position(pt, &self.bpms);

        use graphics::*;

        let textures_map = &self.textures_map;

        let width = args.width as f64;
        let height = args.height as f64;

        // drawable objects
        let mut drawings = vec![];
        for (key, objects) in &self.objects_by_key {
            let start = *self.obj_index_by_key.get(key).unwrap();
            let mut next_start = start;
            for draw in &objects[start..objects.len()] {
                let y = (draw.y - self.y_offset) * self.speed;
                let y = height - y;

                if y > height {
                    next_start += 1;
                } else {
                    drawings.push(DrawInfo { x: draw.x, y: y - NOTES_HEIGHT, width: draw.width, height: draw.height, texture_label: draw.texture_label });
                }
                if y < 0.0 {
                    break;
                }
            }
            *self.obj_index_by_key.get_mut(key).unwrap() = next_start;
        }

        let judge_texture = if self.judge_display.show_until <= 0.0 { None } else {
            match self.judge_display.judge {
                Some(Judge::PGREAT) => Some(TextureLabel::JUDGE_PERFECT),
                Some(Judge::GREAT) => Some(TextureLabel::JUDGE_GREAT),
                Some(Judge::GOOD) => Some(TextureLabel::JUDGE_GOOD),
                Some(Judge::BAD) => Some(TextureLabel::JUDGE_BAD),
                Some(Judge::POOR) => Some(TextureLabel::JUDGE_POOR),
                _ => None,
            }
        };

        let pushed_key_set = &self.pushed_key_set;

        gl.draw(args.viewport(), |c, gl| {
            // back ground
            let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, width, height));
            image.draw(&textures_map[&TextureLabel::BACKGROUND], &DrawState::new_alpha(), c.transform, gl);

            // lanes
            let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, SCR_WIDTH + NOTES1_WIDTH * 4.0 + NOTES2_WIDTH * 3.0, height));
            image.draw(&textures_map[&TextureLabel::LANE_BG], &DrawState::new_alpha(), c.transform, gl);

            for pushed_key in pushed_key_set {
                if let Some((x, beam_width, texture_label)) = beam_info(*pushed_key) {
                    let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, beam_width, height - 14f64));
                    image.draw(&textures_map[&texture_label], &DrawState::new_alpha(), c.transform.trans(x, 0f64), gl)
                }
            }

            // drawable objects
            for draw in &drawings {
                let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, draw.width, draw.height));
                image.draw(&textures_map[&draw.texture_label], &DrawState::new_alpha(), c.transform.trans(draw.x, draw.y - draw.height / 2.0), gl);
            }

            // judge
            if let Some(texture_label) = judge_texture {
                let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, width / 3.0, height / 3.0));
                image.draw(&textures_map[&texture_label], &DrawState::new_alpha(), c.transform.trans(width - width / 3.0, height / 2.0), gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        if self.bpm < 1.0 {
            self.bpm = 1.0;
        }
    }

    fn on_key_down(&mut self, key: &Key) {
        let down = match *key {
            Key::Up => {
                self.speed += 0.1;
                None
            }
            Key::Down => {
                self.speed -= 0.1;
                None
            }
            Key::Space => {
                None
            }
            _ => {
                self.key_mapping.get(key).map(|op| *op)
            }
        };

        // judge
        if let Some(note_key) = down {
            let pt = self.get_precise_time();
            if !self.pushed_key_set.contains(&note_key) {
                self.pushed_key_set.insert(note_key);

                while let Some(mut index) = self.judge_index_by_key.get_mut(&note_key) {
                    if let Some(draw) = self.objects_by_key[&note_key].get(*index) {
                        let timing = draw.timing;
                        if pt <= timing + 0.1 {
                            let time_diff = timing - pt;
                            if let Some(judge) = Judge::get_judge(f64::abs(time_diff)) {
                                self.judge_display.update_judge(judge, pt);
                                *index += 1;
                            }
                            if let Some(wav_id) = draw.wav_id {
                                music::play_sound(&wav_id, music::Repeat::Times(0));
                            }
                            break;
                        }
                    }
                    *index += 1;
                }
            }
        }
    }

    pub fn get_precise_time(&mut self) -> f64 {
        if let Some(init_t) = self.init_time {
            (time::precise_time_s() - init_t) as f64
        } else {
            self.init_time = Some(time::precise_time_s());
            0.0f64
        }
    }

    pub fn process_event(&mut self) {
        let pt = self.get_precise_time();
        // process events
        let start = self.event_index;
        for event in &self.events[start..self.events.len()] {
            if event.timing <= pt {
                self.event_index += 1;
                match event.event_type {
                    EventType::ChangeBpm(ref x) => {
                        self.bpm = *x;
                    }
                    EventType::PlaySound(ref snd) => {
                        music::play_sound(&snd.wav_id, music::Repeat::Times(0));
//                        println!("sound: expected = {}, actual = {}", event.timing, pt);
                    }
                }
            } else {
                break;
            }
        }

    }

    fn on_key_up(&mut self, key: &Key) {
        let up = match *key {
            Key::Up => {
                None
            }
            Key::Down => {
                None
            }
            _ => {
                self.key_mapping.get(key)
            }
        };

        if let Some(key) = up {
            self.pushed_key_set.remove(&key);
        }
    }
}

struct Event {
    timing: Time,
    event_type: EventType
}

enum EventType {
    ChangeBpm(f64),
    PlaySound(bms_loader::Sound)
}

#[derive(Clone)]
struct Draw {
    pub timing: Time,
    pub y: f64,
    pub x: f64,
    pub width: f64,
    pub height: f64,
    pub texture_label: TextureLabel,
    pub wav_id: Option<bms_loader::SoundX>

}

struct DrawInfo {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub texture_label: TextureLabel,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureLabel {
    BACKGROUND,
    LANE_BG,
    NOTE_BLUE,
    NOTE_RED,
    NOTE_WHITE,
    JUDGE_PERFECT,
    JUDGE_GREAT,
    JUDGE_GOOD,
    JUDGE_BAD,
    JUDGE_POOR,
    RED_BEAM,
    WHITE_BEAM,
    BLUE_BEAM,
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
enum Judge {
    PGREAT,
    GREAT,
    GOOD,
    BAD,
    POOR,
}

impl Judge {
    fn get_judge(d: Time) -> Option<Judge> {
        if d < 0.1 {
            Some(if d < 0.02 {
                Judge::PGREAT
            } else if d < 0.03 {
                Judge::GREAT
            } else if d < 0.05 {
                Judge::GOOD
            } else if d < 0.08 {
                Judge::BAD
            } else {
                Judge::POOR
            })
        } else {
            None
        }
    }

    fn combo_lasts(judge: Judge) -> bool {
        match judge {
            Judge::PGREAT | Judge::GREAT | Judge::GOOD => true,
            _ => false
        }
    }
}

struct JudgeDisplay {
    judge: Option<Judge>,
    pub show_until: Time,
    count: HashMap<Judge, u32>,
    combo: u32
}

impl JudgeDisplay {
    pub fn new() -> JudgeDisplay {
        JudgeDisplay{judge: None, show_until: 0.0, count: HashMap::new(), combo: 0u32 }
    }

    pub fn update_judge(&mut self, judge: Judge, t: Time) {
        if let Some(prev) = self.judge {
            if Judge::combo_lasts(prev) && Judge::combo_lasts(judge) {
                self.combo += 1;
            } else {
                self.combo = 0;
            }
        }
        *self.count.entry(judge).or_insert(0) += 1;
        self.judge = Some(judge);
        self.show_until = t + 1.0;
    }
}