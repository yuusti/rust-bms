extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate ears;
extern crate rand;
extern crate regex;

use piston::window::WindowSettings;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use std::path::Path;

use ears::{Sound, AudioController};
use std::thread;

mod bms_parser;
mod bms_player;
mod bms_loader;

use bms_player::Textures;

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

    // load textures here temporarily
    let textures = bms_player::Textures {
        background: Texture::from_path(Path::new("resource/background.png")).unwrap(),
        lane_bg: Texture::from_path(Path::new("resource/lane_bg.png")).unwrap(),
        note_blue: Texture::from_path(Path::new("resource/note_blue.png")).unwrap(),
        note_red: Texture::from_path(Path::new("resource/note_red.png")).unwrap(),
        note_white: Texture::from_path(Path::new("resource/note_white.png")).unwrap(),
    };

    let loader = bms_loader::BmsFileLoader::new("example/conflict/_03_conflict.bme");

    use bms_loader::BmsLoader;
    let mut bms_player = bms_player::BmsPlayer::new(
        GlGraphics::new(opengl),
        &textures,
        loader.load(),
        0.0,
        200.0
    );

//    play_sound("resource/loop.wav");
    bms_player.run(&mut window);
}