use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use graphics::rectangle::square;

use std::path::Path;
use bms_loader::{self, Bms, Sound};
use ears;

type Time = f64;

pub struct BmsPlayer<'a> {
    gl: GlGraphics,
    textures: &'a Textures,
    time: Time,
    speed: f64,
    bpm: f64,
    obj_index: usize,
    event_index: usize,
    objects: Vec<Draw<'a>>,
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
        let mut objects = vec![];
        let mut events = vec![];
        for sound in bms.sounds {
            if assigned_key(sound.key) {
                if let Some((x, width, texture)) = note_info(&textures, sound.key) {
                    objects.push(Draw { key: Some(sound.key), timing: sound.timing, x: x, width: width, height: NOTES_HEIGHT, texture: &texture });
                }
            } else {
                events.push(Event { timing: sound.timing, event_type: EventType::PlaySound(sound) });
            }
        }

        for bar in bms.bars.iter() {
            objects.push(Draw { key: None, timing: *bar, x: 0.0, width: 1000.0, height: BAR_HEIGHT, texture: &textures.background });
        }

        for bpm in bms.bpms.iter() {
            events.push(Event { timing: bpm.timing, event_type: EventType::ChangeBpm(bpm.bpm)});
        }

        objects.sort_by(|a, b| a.timing.partial_cmp(&b.timing).unwrap());
        events.sort_by(|a, b| a.timing.partial_cmp(&b.timing).unwrap());


        BmsPlayer {
            gl: gl,
            textures: textures,
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

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let background = &self.textures.background;
        let lane_bg = &self.textures.lane_bg;

        let width = args.width as f64;
        let height = args.height as f64;

        // drawable objects
        let mut drawings = vec![];
        let start = self.obj_index;
        for draw in &self.objects[start..self.objects.len()] {
            let y = height - Self::calc_pos(draw.timing, self.time, self.bpm, self.speed);
            drawings.push(DrawInfo { x: draw.x, y: y, width: draw.width, height: draw.height, texture: draw.texture });

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
            let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, width, height));
            image.draw(background, &DrawState::new_alpha(), c.transform, gl);

            // lanes
            let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, SCR_WIDTH + NOTES1_WIDTH * 4.0 + NOTES2_WIDTH * 3.0, height));
            image.draw(lane_bg, &DrawState::new_alpha(), c.transform, gl);

            // drawable objects
            for draw in &drawings {
                let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, draw.width, draw.height));
                image.draw(draw.texture, &DrawState::new_alpha(), c.transform.trans(draw.x, draw.y), gl);
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
struct Draw<'a> {
    pub key: Option<bms_loader::Key>,
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