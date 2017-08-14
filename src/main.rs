extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate ears;
extern crate rand;
extern crate regex;
extern crate music;
extern crate sdl2;
extern crate time;
extern crate image;
extern crate ffmpeg;

use piston::event_loop::*;
use piston::input::*;
use graphics::*;
use piston::window::WindowSettings;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use std::path::Path;

use ears::{Sound, AudioController};
use std::thread;
use sdl2::mixer;
use std::env;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::collections::HashMap;

mod bms_parser;
mod bms_player;
mod bms_loader;

fn main() {
    println!("Start main() at {}", time::precise_time_s());
    let opengl = OpenGL::V3_2;

    let script_path = env::args().nth(1).expect("pass script path to first argument");
    let loader = bms_loader::BmsFileLoader::new(&script_path);

    use bms_loader::BmsLoader;

    music::start::<bms_loader::MusicX, bms_loader::SoundX, _>(|| {
        mixer::allocate_channels(256);

        let mut window: Window = WindowSettings::new(
            "rust bms",
            [800, 600]
        )
            .opengl(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        let mut events = Events::new(EventSettings::new());
        let mut gl = GlGraphics::new(opengl);

        println!("Start loading at {}", time::precise_time_s());

        let loading = Texture::from_path(Path::new("resource/loading.png")).unwrap();
        const BG_COLOR: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
        let mut cnt = 0;
        while let Some(e) = events.next(&mut window) {
            if let Some(r) = e.render_args() {
                gl.draw(r.viewport(), |c, gl| {
                    clear(BG_COLOR, gl);
                    let w = r.width as f64;
                    let h = r.height as f64;
                    let image = Image::new().rect(rectangle::rectangle_by_corners(0.0, 0.0, w / 2.0, h / 2.0));
                    image.draw(&loading, &DrawState::new_alpha(), c.transform.trans(w / 4.0, h / 4.0), gl);
                });
                cnt += 1;
                if cnt > 1 {
                    break;
                }
            }
        }

        let mut textures = HashMap::new();
        use bms_player::TextureLabel;
        textures.insert(TextureLabel::BACKGROUND, Texture::from_path(Path::new("resource/background.png")).unwrap());
        textures.insert(TextureLabel::LANE_BG, Texture::from_path(Path::new("resource/lane_bg.png")).unwrap());
        textures.insert(TextureLabel::NOTE_BLUE, Texture::from_path(Path::new("resource/note_blue.png")).unwrap());
        textures.insert(TextureLabel::NOTE_RED, Texture::from_path(Path::new("resource/note_red.png")).unwrap());
        textures.insert(TextureLabel::NOTE_WHITE, Texture::from_path(Path::new("resource/note_white.png")).unwrap());
        textures.insert(TextureLabel::JUDGE_PERFECT, Texture::from_path(Path::new("resource/judge_perfect.png")).unwrap());
        textures.insert(TextureLabel::JUDGE_GREAT, Texture::from_path(Path::new("resource/judge_great.png")).unwrap());
        textures.insert(TextureLabel::JUDGE_GOOD, Texture::from_path(Path::new("resource/judge_good.png")).unwrap());
        textures.insert(TextureLabel::JUDGE_BAD, Texture::from_path(Path::new("resource/judge_bad.png")).unwrap());
        textures.insert(TextureLabel::JUDGE_POOR, Texture::from_path(Path::new("resource/judge_poor.png")).unwrap());
        textures.insert(TextureLabel::RED_BEAM, Texture::from_path(Path::new("resource/redbeam.png")).unwrap());
        textures.insert(TextureLabel::WHITE_BEAM, Texture::from_path(Path::new("resource/whitebeam.png")).unwrap());
        textures.insert(TextureLabel::BLUE_BEAM, Texture::from_path(Path::new("resource/bluebeam.png")).unwrap());

        let mut bms_player = bms_player::BmsPlayer::new(
            textures,
            loader.load(),
            0.0,
            1.0
        );

        bms_player.run(&mut window, &mut gl);
    });
}