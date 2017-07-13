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

type Time = f64;

pub struct BmsPlayer<'a> {
    gl: GlGraphics,
    textures: &'a Textures,
    time: Time,
    speed: f64,
    bpm: f64,
    obj_index_by_key: HashMap<bms_loader::Key, usize>,
    event_index: usize,
    objects_by_key: HashMap<bms_loader::Key, Vec<Draw<'a>>>,
    events: Vec<Event<'a>>,
    judge_index_by_key: HashMap<bms_loader::Key, usize>,
    pushed_key_set: HashSet<bms_loader::Key>,
}

#[inline]
fn note_info(textures: &Textures, key: bms_loader::Key) -> Option<(f64, f64, &Texture)> {
    // x pos, size, color
    use bms_loader::Key;
    let x = key as u8;

    match key {
        Key::P1_KEY1 | Key::P1_KEY3 | Key::P1_KEY5 | Key::P1_KEY7 => {
            Some((SCR_WIDTH + NOTES1_WIDTH * (x / 2) as f64 + NOTES2_WIDTH * ((x - 1) / 2) as f64 + OFFSET, NOTES1_WIDTH - OFFSET * 2.0, &textures.note_white))
        }
        Key::P1_KEY2 | Key::P1_KEY4 | Key::P1_KEY6 => {
            Some((SCR_WIDTH + NOTES1_WIDTH * (x / 2) as f64 + NOTES2_WIDTH * ((x - 1) / 2) as f64 + OFFSET, NOTES2_WIDTH - OFFSET * 2.0, &textures.note_blue))
        }
        Key::P1_SCRATCH => {
            Some((OFFSET, SCR_WIDTH - OFFSET * 2.0, &textures.note_red))
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

impl<'a> BmsPlayer<'a> {
    pub fn new(
        gl: GlGraphics,
        textures: &'a Textures,
        bms: Bms<'a>,
        time: Time,
        speed: f64,
    ) -> BmsPlayer<'a> {
        let mut objects_by_key = HashMap::new();
        for key in bms_loader::Key::visible_keys() {
            objects_by_key.insert(key, vec![]);
        }

        let mut events = vec![];
        for sound in bms.sounds {
            if bms_loader::Key::visible_keys().contains(&sound.key) {
                if let Some((x, width, texture)) = note_info(&textures, sound.key) {
                    objects_by_key.get_mut(&sound.key).unwrap().push(Draw { timing: sound.timing, x: x, width: width, height: NOTES_HEIGHT, texture: &texture });
                }
            } else {
                events.push(Event { timing: sound.timing, event_type: EventType::PlaySound(sound) });
            }
        }

        objects_by_key.insert(bms_loader::Key::BACK_CHORUS, vec![]);
        for bar in bms.bars.iter() {
            objects_by_key.get_mut(&bms_loader::Key::BACK_CHORUS).unwrap().push(Draw { timing: *bar, x: 0.0, width: 1000.0, height: BAR_HEIGHT, texture: &textures.background });
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

        BmsPlayer {
            gl: gl,
            textures: textures,
            time: time,
            speed: speed,
            bpm: 130f64,
            obj_index_by_key: obj_index_by_key.clone(),
            event_index: 0usize,
            objects_by_key: objects_by_key,
            events: events,
            judge_index_by_key: obj_index_by_key.clone(),
            pushed_key_set: HashSet::new()
        }
    }

    pub fn run(&mut self, window: &mut Window) {
        let mut events = Events::new(EventSettings::new());

        while let Some(e) = events.next(window) {
            if let Some(r) = e.render_args() {
                self.render(&r);
            }

            if let Some(u) = e.update_args() {
                self.update(&u);
            }

            if let Some(Button::Keyboard(key)) = e.press_args() {
                self.on_key_down(&key);
            }

            if let Some(Button::Keyboard(key)) = e.release_args() {
                self.on_key_up(&key);
            }
        }
    }

    #[inline]
    fn calc_pos(arrival_time: f64, current_time: f64, bpm: f64, speed: f64) -> f64 {
        (arrival_time - current_time) * bpm / 240f64 * speed
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let background = &self.textures.background;
        let lane_bg = &self.textures.lane_bg;

        let width = args.width as f64;
        let height = args.height as f64;

        // drawable objects
        let mut drawings = vec![];
        for (key, objects) in &self.objects_by_key {
            let start = *self.obj_index_by_key.get(key).unwrap();
            let mut next_start = start;
            for draw in &objects[start..objects.len()] {
                let y = height - NOTES_HEIGHT - Self::calc_pos(draw.timing, self.time, self.bpm, self.speed);

                if y > height {
                    next_start += 1;
                } else {
                    drawings.push(DrawInfo { x: draw.x, y: y, width: draw.width, height: draw.height, texture: draw.texture });
                }
                if y < 0.0 {
                    break;
                }
            }
            *self.obj_index_by_key.get_mut(key).unwrap() = next_start;
        }

        // process events
        let start = self.event_index;
        for event in &self.events[start..self.events.len()] {
            if event.timing <= self.time {
                self.event_index += 1;
                match event.event_type {
                    EventType::ChangeBpm(ref x) => {
                        self.bpm = *x;
                    }
                    EventType::PlaySound(ref snd) => (),
                }
            } else {
                break;
            }
        }

        self.gl.draw(args.viewport(), |c, gl| {
            // back ground
            let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, width, height));
            image.draw(background, &DrawState::new_alpha(), c.transform, gl);

            // lanes
            let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, SCR_WIDTH + NOTES1_WIDTH * 4.0 + NOTES2_WIDTH * 3.0, height));
            image.draw(lane_bg, &DrawState::new_alpha(), c.transform, gl);

            // drawable objects
            for draw in &drawings {
                let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, draw.width, draw.height));
                image.draw(draw.texture, &DrawState::new_alpha(), c.transform.trans(draw.x, draw.y - draw.height / 2.0), gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.time += args.dt;
        if self.bpm < 1.0 {
            self.bpm = 1.0;
        }

        // update judge
    }

    fn on_key_down(&mut self, key: &Key) {
        let down = match *key {
            Key::A => Some(bms_loader::Key::P1_SCRATCH),
            Key::Z => Some(bms_loader::Key::P1_KEY1),
            Key::S => Some(bms_loader::Key::P1_KEY2),
            Key::X => Some(bms_loader::Key::P1_KEY3),
            Key::D => Some(bms_loader::Key::P1_KEY4),
            Key::C => Some(bms_loader::Key::P1_KEY5),
            Key::F => Some(bms_loader::Key::P1_KEY6),
            Key::V => Some(bms_loader::Key::P1_KEY7),
            Key::Up => {
                self.speed += 10.0;
                None
            }
            Key::Down => {
                self.speed -= 10.0;
                None
            }
            _ => None,
        };

        // judge
        if let Some(note_key) = down {
            if !self.pushed_key_set.contains(&note_key) {
                self.pushed_key_set.insert(note_key);
                println!("key {:?} pressed", note_key);

                while let Some(mut index) = self.judge_index_by_key.get_mut(&note_key) {
                    if let Some(draw) = self.objects_by_key[&note_key].get(*index) {
                        let timing = draw.timing;
                        if self.time <= timing + 0.1 {
                            let time_diff = timing - self.time;
                            println!("timing={} ({})", time_diff, if time_diff < 0.0 { "SLOW" } else { "FAST" });
                            if let Some(judge) = Judge::get_judge(f64::abs(time_diff)) {
                                println!("{:?}", judge);
                                *index += 1;
                            }
                            break;
                        }
                    }
                    *index += 1;
                }
            }
        }

    }

    fn on_key_up(&mut self, key: &Key) {
        let up = match *key {
            Key::A => Some(bms_loader::Key::P1_SCRATCH),
            Key::Z => Some(bms_loader::Key::P1_KEY1),
            Key::S => Some(bms_loader::Key::P1_KEY2),
            Key::X => Some(bms_loader::Key::P1_KEY3),
            Key::D => Some(bms_loader::Key::P1_KEY4),
            Key::C => Some(bms_loader::Key::P1_KEY5),
            Key::F => Some(bms_loader::Key::P1_KEY6),
            Key::V => Some(bms_loader::Key::P1_KEY7),
            Key::Up => {
                self.speed += 10.0;
                None
            }
            Key::Down => {
                self.speed -= 10.0;
                None
            }
            _ => None,
        };

        if let Some(key) = up {
            self.pushed_key_set.remove(&key);
        }
    }
}

struct Event<'a> {
    timing: Time,
    event_type: EventType<'a>
}

enum EventType<'a> {
    ChangeBpm(f64),
    PlaySound(bms_loader::Sound<'a>)
}

#[derive(Clone)]
struct Draw<'a> {
    pub timing: Time,
    pub x: f64,
    pub width: f64,
    pub height: f64,
    pub texture: &'a Texture,
}

struct DrawInfo<'a> {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub texture: &'a Texture,
}

pub struct Textures {
    pub background: Texture,
    pub lane_bg: Texture,
    pub note_blue: Texture,
    pub note_red: Texture,
    pub note_white: Texture,
}


#[derive(Debug)]
enum Judge {
    PGREAT,
    GREAT,
    GOOD,
    BAD,
    POOR,
}

impl Judge {
    fn get_judge(d: f64) -> Option<Judge> {
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
}