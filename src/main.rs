extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate ears;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use graphics::rectangle::square;
use std::path::Path;

pub struct App {
    gl: GlGraphics,
    notes_texture: Texture,
    background_texture: Texture,
    judge: Texture,
    chart: Chart,
    time: f64,
    sp: f64
}

impl App {
    fn render(&mut self, args: &RenderArgs, b: &mut i32) {
        use graphics::*;

        let background = &self.background_texture;
        let bars = &self.chart.bars;
        const WHITE: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        let sp = self.sp;
        let time = self.time;
        let bpm = self.chart.bpm;
        self.gl.draw(args.viewport(), |c, gl| {
            // back ground
            // TODO: use rectangle
            let image = Image::new().rect(square(0.0, 0.0, args.width as f64));
            image.draw(background, &DrawState::new_alpha(), c.transform, gl);

            // TODO: bar line
            let height = args.height as f64;
            for i in 0..bars.len() + 1000 {
                let i = i as f64;
                let sp = sp * height;
                let h = height + time * sp - i * sp * 240.0 / bpm;
                rectangle(WHITE, rectangle::rectangle_by_corners(0.0, 0.0, args.width as f64, 5.0),
                          c.transform.trans(0.0, h), gl);
            }

            // TODO: notes
            for bar in bars {
            }

            // TODO: judge

        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.time += args.dt;
    }

    fn on_key_down(&mut self, key: &Key) {
        self.sp += 1.0;
    }

    fn on_key_up(&mut self, key: &Key) {}

    fn play_sound() {
        // TODO:
    }
}

use ears::{Sound, AudioController};
use std::thread;

fn play_sound(path: &'static str) {
    print!("{}", path);
    thread::spawn(move || {
        match ears::Sound::new(&path) {
            Some(mut snd) => {
                snd.play();
                while snd.is_playing() {}
            }
            None => {
                println!("failed");
            }
        }
    });
}

pub struct Bar {
    num: i32,
    ch: i32,
    obj: String,
}

pub struct Chart {
    bpm: f64,
    bars: Vec<Bar>,
}

fn test_chart() -> Chart {
    Chart {
        bpm: 130.0,
        bars: vec![
            Bar {
                num: 1,
                ch: 11,
                obj: "01".to_string(),
            },
            Bar {
                num: 2,
                ch: 11,
                obj: "01".to_string(),
            },
            Bar {
                num: 3,
                ch: 11,
                obj: "01".to_string(),
            }
        ]
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new(
        "rust bms",
        [800, 600]
    )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let texture = Texture::from_path(Path::new("resource/a.png")).unwrap();
    let bg = Texture::from_path(Path::new("resource/b.png")).unwrap();
    let x = Texture::from_path(Path::new("resource/c.png")).unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        notes_texture: texture,
        background_texture: bg,
        judge: x,
        chart: test_chart(),
        time: 0.0,
        sp: 1.0,
    };

    let mut events = Events::new(EventSettings::new());

    play_sound("resource/loop.wav");
    let mut xx = 0;
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r, &mut xx);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            app.on_key_down(&key);
        }

        if let Some(Button::Keyboard(key)) = e.release_args() {
            app.on_key_up(&key);
        }
    }
}