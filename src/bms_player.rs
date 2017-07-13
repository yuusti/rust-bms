use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use graphics::rectangle::square;

use bms_loader::{self, Bms, Sound};
use ears;

type Time = f64;

pub struct BmsPlayer<'a> {
    gl: GlGraphics,
    notes_texture: Texture,
    background_texture: Texture,
    judge: Texture,
    time: Time,
    speed: f64,
    bpm: f64,
    obj_index: usize,
    event_index: usize,
    objects: Vec<Draw>,
    events: Vec<Event<'a>>,
}

#[derive(Copy, Clone)]
enum Color {
    RED,
    WHITE,
    BLUE,
    BLACK,
    GREY,
}

impl Color {
    pub fn value(&self) -> [f32; 4] {
        match *self {
            Color::RED => [1.0, 0.0, 0.0, 1.0],
            Color::WHITE => [1.0, 1.0, 1.0, 1.0],
            Color::BLUE => [0.0, 0.0, 1.0, 1.0],
            Color::BLACK => [0.0, 0.0, 0.0, 1.0],
            Color::GREY => [0.5, 0.5, 0.5, 1.0],
        }
    }
}

fn assigned_key(key: bms_loader::Key) -> bool {
    (key as u8) < 8
}

#[inline]
fn note_info(key: bms_loader::Key) -> Option<(f64, f64, Color)> {
    // x pos, size, color
    use bms_loader::Key;
    let x = key as u8;
    const SCR: f64 = 100f64;
    const KEY: f64 = 60f64;

    match key {
        Key::P1_KEY1 | Key::P1_KEY3 | Key::P1_KEY5 | Key::P1_KEY7 => {
            Some((SCR + KEY * ((x - 1) as f64), KEY, Color::WHITE))
        }
        Key::P1_KEY2 | Key::P1_KEY4 | Key::P1_KEY6 => {
            Some((SCR + KEY * ((x - 1) as f64), KEY, Color::BLUE))
        }
        Key::P1_SCRATCH => {
            Some((0f64, SCR, Color::RED))
        }
        _ => None,
    }
}

const NOTES_HEIGHT: f64 = 5.0;
const BAR_HEIGHT: f64 = 1.0;

impl<'a> BmsPlayer<'a> {
    pub fn new(
        gl: GlGraphics,
        notes_texture: Texture,
        background_texture: Texture,
        judge: Texture,
        bms: Bms<'a>,
        time: Time,
        speed: f64,
    ) -> BmsPlayer<'a> {
        let mut objects = vec![];
        let mut events = vec![];
        for sound in bms.sounds {
            if assigned_key(sound.key) {
                if let Some((x, width, color)) = note_info(sound.key) {
                    objects.push(Draw { key: Some(sound.key), timing: sound.timing, x: x, width: width, height: NOTES_HEIGHT, color: color });
                }
            } else {
                events.push(Event { timing: sound.timing, event_type: EventType::PlaySound(sound) });
            }
        }

        for bar in bms.bars.iter() {
            objects.push(Draw { key: None, timing: *bar, x: 0.0, width: 1000.0, height: BAR_HEIGHT, color: Color::GREY });
        }

        for bpm in bms.bpms.iter() {
            events.push(Event { timing: bpm.timing, event_type: EventType::ChangeBpm(bpm.bpm)});
        }

        objects.sort_by(|a, b| a.timing.partial_cmp(&b.timing).unwrap());
        events.sort_by(|a, b| a.timing.partial_cmp(&b.timing).unwrap());

        BmsPlayer {
            gl: gl,
            notes_texture: notes_texture,
            background_texture: background_texture,
            judge: judge,
            time: time,
            speed: speed,
            bpm: 130f64,
            obj_index: 0usize,
            event_index: 0usize,
            objects: objects,
            events: events,
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

    fn calc_bar_pos(&self, bars: &[f64], judge_line: f64) -> Vec<f64> {
        bars.into_iter().map(|t|
            judge_line - Self::calc_pos(*t, self.time, self.bpm, self.speed)
        ).collect()
    }

    fn calc_note_pos(&self, sounds: &[Sound], judge_line: f64) -> Vec<(bms_loader::Key, f64)> {
        sounds.into_iter().map(|s| {
            (s.key, judge_line - Self::calc_pos(s.timing, self.time, self.bpm, self.speed))
        }).collect()
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let background = &self.background_texture;

        let height = args.height as f64;

        // drawable objects
        let mut drawings = vec![];
        let start = self.obj_index;
        for draw in &self.objects[start..self.objects.len()] {
            let y = height - Self::calc_pos(draw.timing, self.time, self.bpm, self.speed);
            drawings.push(DrawInfo { x: draw.x, y: y, width: draw.width, height: draw.height, color: draw.color });

            if y > height {
                self.obj_index += 1;
            }
            if y < 0.0 {
                break;
            }
        }

        let events = &self.events;
        let event_index = self.event_index;


        self.gl.draw(args.viewport(), |c, gl| {
            // back ground
            // TODO: use rectangle
            let image = Image::new().rect(square(0.0, 0.0, args.width as f64));
            image.draw(background, &DrawState::new_alpha(), c.transform, gl);

            // drawable objects
            for draw in &drawings {
                rectangle(draw.color.value(), rectangle::rectangle_by_corners(0.0, 0.0, draw.width, draw.height),
                          c.transform.trans(draw.x, draw.y), gl);
            }

            // events
            for event in &events[event_index..events.len()] {}

            // TODO: judge?
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.time += args.dt;
    }

    fn on_key_down(&mut self, key: &Key) {
        self.speed += 10.0;
    }

    fn on_key_up(&mut self, key: &Key) {}
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
struct Draw {
    pub key: Option<bms_loader::Key>,
    pub timing: Time,
    pub x: f64,
    pub width: f64,
    pub height: f64,
    pub color: Color
}

struct DrawInfo {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Color
}