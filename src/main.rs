extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate ears;
extern crate rand;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use graphics::rectangle::square;
use std::path::Path;

use ears::{Sound, AudioController};
use std::thread;

mod bms_player;
mod bms_loader;

#[allow(dead_code)]
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
    let bg = Texture::from_path(Path::new("resource/a.png")).unwrap();
    let x = Texture::from_path(Path::new("resource/a.png")).unwrap();

    let loader = bms_loader::FixtureLoader::new();

    use bms_loader::BmsLoader;
    let mut bms_player = bms_player::BmsPlayer::new(
        GlGraphics::new(opengl),
        texture,
        bg,
        x,
        loader.load(),
        0.0,
        200.0
    );

//    play_sound("resource/loop.wav");
    bms_player.run(&mut window);
}