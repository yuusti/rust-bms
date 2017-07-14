extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate ears;
extern crate rand;
extern crate regex;
extern crate music;
extern crate sdl2;

use piston::window::WindowSettings;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use std::path::Path;

use ears::{Sound, AudioController};
use std::thread;
use sdl2::mixer;
use std::env;

mod bms_parser;
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

    // load textures here temporarily
    let textures = bms_player::Textures {
        background: Texture::from_path(Path::new("resource/background.png")).unwrap(),
        lane_bg: Texture::from_path(Path::new("resource/lane_bg.png")).unwrap(),
        note_blue: Texture::from_path(Path::new("resource/note_blue.png")).unwrap(),
        note_red: Texture::from_path(Path::new("resource/note_red.png")).unwrap(),
        note_white: Texture::from_path(Path::new("resource/note_white.png")).unwrap(),
        judge_perfect: Texture::from_path(Path::new("resource/judge_perfect.png")).unwrap(),
        judge_great: Texture::from_path(Path::new("resource/judge_great.png")).unwrap(),
        judge_good: Texture::from_path(Path::new("resource/judge_good.png")).unwrap(),
        judge_bad: Texture::from_path(Path::new("resource/judge_bad.png")).unwrap(),
        judge_poor: Texture::from_path(Path::new("resource/judge_poor.png")).unwrap()
    };

    let script_path = env::args().nth(1).expect("pass script path to first argument");
    let loader = bms_loader::BmsFileLoader::new(&script_path);

    use bms_loader::BmsLoader;

    music::start::<bms_loader::MusicX, bms_loader::SoundX, _>(|| {
        mixer::allocate_channels(256);
        let mut bms_player = bms_player::BmsPlayer::new(
            GlGraphics::new(opengl),
            &textures,
            loader.load(),
            0.0,
            1.0
        );

        bms_player.run(&mut window);
    });
}