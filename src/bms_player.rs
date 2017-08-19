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
use std::cmp;

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
    preset_textures: PresetTextures,
    key_mapping: HashMap<Key, bms_loader::Key>,
    bga_textures: Vec<Texture>,
    bga_id: Option<i32>,
    judgerank: JudgeRank,
    state: GameState,

}

#[derive(Eq, PartialEq)]
enum GameState {
    PLAY,
    STOP,
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
const LANE_WIDTH: f64 = SCR_WIDTH + NOTES1_WIDTH * 4.0 + NOTES2_WIDTH * 3.0;

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
    f64::abs(a - b) < 1e-9
}

#[test]
pub fn test_calc_position() {
    let bpms = vec![
        bms_loader::BpmChange { timing: 0f64, bpm: 100f64 },
        bms_loader::BpmChange { timing: 10f64, bpm: 200f64 },
        bms_loader::BpmChange { timing: 20f64, bpm: 400f64 },
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
        preset_textures: PresetTextures,
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
                    objects_by_key.get_mut(&sound.key).unwrap().push(Draw { timing: sound.timing, x: x, y: calc_position(sound.timing, &bms.bpms), width: width, height: NOTES_HEIGHT, texture_label: texture_label, wav_id: Some(sound.wav_id) });
                }
            } else if sound.key == bms_loader::Key::BACK_CHORUS {
                events.push(Event { timing: sound.timing, event_type: EventType::PlaySound(sound) });
            }
        }

        let mut bga_ends: f64 = 0.0;
        for image in bms.bga {
            events.push(Event {timing: image.timing, event_type: EventType::ChangeBga(Some(image.texture_id))});
            if bga_ends < image.timing {
                bga_ends = image.timing;
            }
        }
        events.push(Event {timing: bga_ends + 0.5, event_type: EventType::ChangeBga(None)});

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
        let end_timing = 1.0 + match events.last() {
            Some(event) => event.timing,
            None => 0.0
        };
        events.push(Event {timing: end_timing, event_type: EventType::EndMusic });

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
            preset_textures: preset_textures,
            key_mapping: key_mapping,
            bga_textures: bms.textures,
            bga_id: None,
            judgerank: IIDX_JUDGERANK,
            state: GameState::PLAY
        }
    }

    pub fn run(&mut self, window: &mut Window, gl: &mut GlGraphics) {
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

            if self.state == GameState::STOP {
                break;
            }
        }
    }

    fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics) {
        let pt = self.get_precise_time();
        self.y_offset = calc_position(pt, &self.bpms);

        use graphics::*;

        let digits = &self.preset_textures.digits;
        let textures_map = &self.preset_textures.lane_components;

        let width = args.width as f64;
        let height = args.height as f64;

        // drawable objects
        let mut drawings = vec![];
        for (key, objects) in &self.objects_by_key {
            let start = *self.obj_index_by_key.get(key).unwrap();
            let judge_consumed = *self.judge_index_by_key.get(key).unwrap_or(&0usize);
            let start = cmp::max(start, judge_consumed);

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

        let judge_texture = if pt <= self.judge_display.show_until {
            let mut x = self.judge_display.combo;
            let digits = if x > 0 {
                let mut v = Vec::new();
                while x > 0 {
                    v.push(x % 10);
                    x /= 10;
                }
                v.reverse();
                v
            } else {
                Vec::new()
            };

            if let Some(judge) = self.judge_display.judge {
                Some((match judge {
                    Judge::PGREAT => TextureLabel::JUDGE_PERFECT,
                    Judge::GREAT => TextureLabel::JUDGE_GREAT,
                    Judge::GOOD => TextureLabel::JUDGE_GOOD,
                    Judge::BAD => TextureLabel::JUDGE_BAD,
                    Judge::POOR => TextureLabel::JUDGE_POOR,
                }, digits))
            } else {
                None
            }
        } else { None };

        let pushed_key_set = &self.pushed_key_set;
        let bga_map = &self.bga_textures;
        let bga = self.bga_id;

        gl.draw(args.viewport(), |mut c, gl| {
            // back ground
            let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, width, height));
            image.draw(&textures_map[&TextureLabel::BACKGROUND], &DrawState::new_alpha(), c.transform, gl);

            // lanes
            let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, LANE_WIDTH, height));
            image.draw(&textures_map[&TextureLabel::LANE_BG], &DrawState::new_alpha(), c.transform, gl);

            // beams
            for pushed_key in pushed_key_set {
                if let Some((x, beam_width, texture_label)) = beam_info(*pushed_key) {
                    let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, beam_width, height - 14f64));
                    image.draw(&textures_map[&texture_label], &DrawState::new_alpha(), c.transform.trans(x, 0f64), gl)
                }
            }

            // notes and bars
            for draw in &drawings {
                let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, draw.width, draw.height));
                image.draw(&textures_map[&draw.texture_label], &DrawState::new_alpha(), c.transform.trans(draw.x, draw.y - draw.height / 2.0), gl);
            }

            // bga
            bga.map(|id| {
                let size = if width - LANE_WIDTH < height { width - LANE_WIDTH } else { height };
                let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, size, size));
                image.draw(&bga_map[id as usize], &DrawState::new_alpha(), c.transform.trans(LANE_WIDTH, 0f64), gl)
            });

            // judge
            if let Some((texture_label, ref combo_digits)) = judge_texture {
                let (w, h) = textures_map[&texture_label].get_size();
                let scale = 2.0;
                let w = w as f64 * scale;
                let h = h as f64 * scale;
                let mut combined = CombinedTexture::new();
                combined.add(TextureDisplay {texture: &textures_map[&texture_label], w: w, h: h});

                for &digit in combo_digits {
                    let digit = &digits[digit as usize];
                    // using the width of '0' for all digits to be monospaced...
                    let (dw, dh) = digits[0usize].get_size();
                    let scale = h as f64 / dh as f64;
                    let dw = dw as f64 * scale;
                    let dh = dh as f64 * scale;

                    combined.add(TextureDisplay {texture: digit, w: dw, h: dh});
                }

                let lx = (LANE_WIDTH - combined.get_w()) / 2.0;
                combined.draw(&mut c, gl, lx, 0.7 * height as f64);
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
            Key::Escape => {
                self.state = GameState::STOP;
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
                    match self.objects_by_key[&note_key].get(*index) {
                        Some(draw) => {
                            let timing = draw.timing;
                            if pt <= timing + 0.1 {
                                let time_diff = timing - pt;
                                if let Some(judge) = self.judgerank.get_judge(f64::abs(time_diff)) {
                                    self.judge_display.update_judge(judge, pt);
                                    *index += 1;
                                }
                                if let Some(wav_id) = draw.wav_id {
                                    music::play_sound(&wav_id, music::Repeat::Times(0));
                                }
                                break;
                            }
                        }
                        None => break
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
                    EventType::ChangeBga(ref id) => {
                        self.bga_id = *id;
                    }
                    EventType::EndMusic => {
                        self.state = GameState::STOP;
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
    PlaySound(bms_loader::Sound),
    ChangeBga(Option<i32>),
    EndMusic
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

pub struct PresetTextures {
    pub lane_components: HashMap<TextureLabel, Texture>,
    pub digits: Vec<Texture>,
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
    fn combo_lasts(judge: Judge) -> bool {
        match judge {
            Judge::PGREAT | Judge::GREAT | Judge::GOOD => true,
            _ => false
        }
    }
}

struct JudgeRank {
    pgreat: f64,
    great: f64,
    good: f64,
    bad: f64,
    poor: f64,
}

impl JudgeRank {
    fn get_judge(&self, d: Time) -> Option<Judge> {
        if d < self.poor {
            Some(if d < self.pgreat {
                Judge::PGREAT
            } else if d < self.great {
                Judge::GREAT
            } else if d < self.good {
                Judge::GOOD
            } else if d < self.bad {
                Judge::BAD
            } else {
                Judge::POOR
            })
        } else {
            None
        }
    }
}

const IIDX_JUDGERANK: JudgeRank = JudgeRank { pgreat: 0.02, great: 0.04, good: 0.105, bad: 0.15, poor: 0.2 };

struct JudgeDisplay {
    judge: Option<Judge>,
    pub show_until: Time,
    count: HashMap<Judge, u32>,
    combo: u32
}

impl JudgeDisplay {
    pub fn new() -> JudgeDisplay {
        JudgeDisplay { judge: None, show_until: 0.0, count: HashMap::new(), combo: 0u32 }
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

struct TextureDisplay<'a> {
    texture: &'a Texture,
    w: f64,
    h: f64,
}

struct CombinedTexture<'a> {
    texture_displays: Vec<TextureDisplay<'a>>,
    total_w: f64,
}

use graphics::*;
impl <'a>CombinedTexture<'a> {
    pub fn new() -> CombinedTexture<'a> {
        CombinedTexture {texture_displays: vec![], total_w: 0.0}
    }

    pub fn add(&mut self, texture: TextureDisplay<'a>) {
        self.total_w += texture.w;
        self.texture_displays.push(texture);
    }

    pub fn get_w(&self) -> f64 {
        self.total_w
    }

    pub fn draw(&self, c: &mut Context, gl: &mut GlGraphics, x: f64, y: f64) {
        let mut w = 0.0;
        for t in &self.texture_displays {
            let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, t.w, t.h));
            image.draw(t.texture, &DrawState::new_alpha(), c.transform.trans(x + w, y), gl);
            w += t.w;
        }
    }
}