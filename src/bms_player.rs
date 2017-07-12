
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use graphics::rectangle::square;

use bms_loader::Chart;

pub struct BmsPlayer {
    gl: GlGraphics,
    notes_texture: Texture,
    background_texture: Texture,
    judge: Texture,
    chart: Chart,
    time: f64,
    sp: f64
}

impl BmsPlayer {
    pub fn new(
        gl: GlGraphics,
        notes_texture: Texture,
        background_texture: Texture,
        judge: Texture,
        chart: Chart,
        time: f64,
        sp: f64
    ) -> BmsPlayer {
        BmsPlayer {
            gl: gl,
            notes_texture: notes_texture,
            background_texture: background_texture,
            judge: judge,
            chart: chart,
            time: time,
            sp: sp,
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

    fn render(&mut self, args: &RenderArgs) {
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
}
