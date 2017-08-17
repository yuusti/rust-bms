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
extern crate walkdir;

use piston::event_loop::*;
use piston::input::*;
use graphics::*;
use piston::window::WindowSettings;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use opengl_graphics::glyph_cache::GlyphCache;
use std::path::{Path, PathBuf};

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

use bms_parser::BmsParser;
use bms_player::TextureLabel;

use walkdir::{DirEntry, WalkDir, WalkDirIterator};

fn main() {
    println!("Start main() at {}", time::precise_time_s());
    let opengl = OpenGL::V3_2;
    music::start::<bms_loader::MusicX, bms_loader::SoundX, _>(|| {
        mixer::allocate_channels(256);

        let mut window: Window = WindowSettings::new(
            "rust bms",
            [800, 600]
        )
            .opengl(opengl)
            .build()
            .unwrap();
        let mut gl = GlGraphics::new(opengl);

        match env::args().nth(1) {
            Some(path) => play_bms(&mut window, &mut gl, path),
            None => music_selection(&mut window, &mut gl)
        }
    });
}

fn music_selection(mut window: &mut Window, mut gl: &mut GlGraphics) {
    show_loading(&mut window, &mut gl);

    let ref mut glyphs = GlyphCache::new("resource/font/rounded-mplus-1p-regular.ttf")
        .expect("Could not load font");

    let bms_base = env::current_dir().unwrap().join("bms");
    let bms_paths: Vec<PathBuf> = WalkDir::new(bms_base.clone()).into_iter().filter_map(|e| e.ok()).map(|entry| {
        if is_bms(&entry) {
            Some(entry.path().to_path_buf())
        } else {
            None
        }
    }).flat_map(|x| match x {
        Some(e) => vec![e],
        None => vec![],
    }).collect();

    assert!(bms_paths.len() > 0, "place bms files under following directory: {}", bms_base.to_str().unwrap());

    let path_title: Vec<(PathBuf, String)> = bms_paths.into_iter().map(|path_buf| {
        let bms_script = bms_parser::BmsFileParser{path: path_buf.to_str().unwrap().to_string()}.parse();
        println!("{}", bms_script.headers()["TITLE"]);
        (path_buf, bms_script.headers()["TITLE"].clone())
    }).collect();

    const BG_COLOR: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
    const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
    const FONT_SIZE: u32 = 50;

    let mut cur = 0;
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(window) {
        if let Some(r) = e.render_args() {
            gl.draw(r.viewport(), |c, gl| {
                clear(BG_COLOR, gl);
                let w = r.width as f64;
                let h = r.height as f64;

                let (ref path, ref title) = path_title[cur];
                rectangle(GREEN, rectangle::rectangle_by_corners(0.0, 0.0, w, FONT_SIZE as f64), c.transform.trans(0.0, h / 2.0 - FONT_SIZE as f64 * 0.9), gl);
                Text::new(FONT_SIZE).draw(&title, glyphs, &DrawState::new_alpha(), c.transform.trans(0.0, h / 2.0), gl);

                let display_num = 5;
                for i in 1..display_num + 1 {
                    let (ref path, ref title) = path_title[(cur + i) % path_title.len()];
                    Text::new(FONT_SIZE).draw(&title, glyphs, &DrawState::new_alpha(), c.transform.trans(0.0, h / 2.0 - i as f64 * FONT_SIZE as f64), gl);

                    let (ref path, ref title) = path_title[(cur + path_title.len() * 100 - i) % path_title.len()];
                    Text::new(FONT_SIZE).draw(&title, glyphs, &DrawState::new_alpha(), c.transform.trans(0.0, h / 2.0 + i as f64 * FONT_SIZE as f64), gl);
                }

            });
        }
        if let Some(Button::Keyboard(key)) = e.press_args() {
            let down = match key {
                Key::Up => {
                    cur += 1;
                    cur %= path_title.len();
                }
                Key::Down => {
                    cur += path_title.len() - 1;
                    cur %= path_title.len();
                }
                Key::Return => {
                    play_bms(&mut window, &mut gl, path_title[cur].0.to_str().unwrap().to_string());
                }
                Key::Escape => {
                    break;
                }
                _ => {
                    ()
                }
            };
        }
    }
}

fn is_bms(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.ends_with(".bme"))
        .unwrap_or(false)
}

fn show_loading(mut window: &mut Window, mut gl: &mut GlGraphics) {
    let loading = Texture::from_path(Path::new("resource/loading.png")).unwrap();
    const BG_COLOR: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
    let mut cnt = 0;

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(window) {
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
}

fn play_bms(mut window: &mut Window, mut gl: &mut GlGraphics, script_path: String) {
    show_loading(&mut window, &mut gl);

    let mut textures_map = HashMap::new();
    textures_map.insert(TextureLabel::BACKGROUND, Texture::from_path(Path::new("resource/background.png")).unwrap());
    textures_map.insert(TextureLabel::LANE_BG, Texture::from_path(Path::new("resource/lane_bg.png")).unwrap());
    textures_map.insert(TextureLabel::NOTE_BLUE, Texture::from_path(Path::new("resource/note_blue.png")).unwrap());
    textures_map.insert(TextureLabel::NOTE_RED, Texture::from_path(Path::new("resource/note_red.png")).unwrap());
    textures_map.insert(TextureLabel::NOTE_WHITE, Texture::from_path(Path::new("resource/note_white.png")).unwrap());
    textures_map.insert(TextureLabel::JUDGE_PERFECT, Texture::from_path(Path::new("resource/judge_perfect.png")).unwrap());
    textures_map.insert(TextureLabel::JUDGE_GREAT, Texture::from_path(Path::new("resource/judge_great.png")).unwrap());
    textures_map.insert(TextureLabel::JUDGE_GOOD, Texture::from_path(Path::new("resource/judge_good.png")).unwrap());
    textures_map.insert(TextureLabel::JUDGE_BAD, Texture::from_path(Path::new("resource/judge_bad.png")).unwrap());
    textures_map.insert(TextureLabel::JUDGE_POOR, Texture::from_path(Path::new("resource/judge_poor.png")).unwrap());
    textures_map.insert(TextureLabel::RED_BEAM, Texture::from_path(Path::new("resource/redbeam.png")).unwrap());
    textures_map.insert(TextureLabel::WHITE_BEAM, Texture::from_path(Path::new("resource/whitebeam.png")).unwrap());
    textures_map.insert(TextureLabel::BLUE_BEAM, Texture::from_path(Path::new("resource/bluebeam.png")).unwrap());

    let loader = bms_loader::BmsFileLoader::new(&script_path);

    use bms_loader::BmsLoader;
    println!("Start loading at {}", time::precise_time_s());
    let mut bms_player = bms_player::BmsPlayer::new(
        textures_map,
        loader.load(),
        0.0,
        1.0
    );

    bms_player.run(&mut window, &mut gl);
}