#![feature(cell_update)]

use macroquad::prelude::*;
use macroquad::color::Color;
use std::cell::Cell;
use std::cmp::*;
use rand_xoshiro::Xoshiro128StarStar;
use rand_core::SeedableRng;
use rand_core::RngCore;
use macroquad::input::*;
use std::time::{SystemTime, UNIX_EPOCH};

struct Game {

}

pub enum LevelName {
    L1 = 0,
}

#[derive(PartialEq,Eq,Debug,Clone)]
pub struct Optic {
    x_from: i16,
    w_from: i16,
    x_to: i16,
    w_to: i16,
    y: Cell<i16>,
}

impl Optic {
    fn get_left_complement(&self) -> Optic {
        Optic {
            x_from: 0,
            w_from: self.x_from,
            x_to: 0,
            w_to: self.x_to,
            y: Cell::new(self.y.get())
        }
    }

    fn get_right_complement(&self) -> Optic {
        let start_from = self.x_from + self.w_from;
        let start_to = self.x_to + self.w_to;
        Optic {
            x_from: start_from,
            w_from: W - start_from,
            x_to: start_to,
            w_to: W - start_to,
            y: Cell::new(self.y.get())
        }
    }

    fn step(&self) {
        self.y.update(|y| y + SPEED);
    }

    fn shift(&self, shift: i16) -> Optic {
        let mut x_from = self.x_from + shift;
        let mut x_to = self.x_to + shift;

        // left border
        if x_to < 0 {
            x_from -= x_to;
            x_to = 0;
        }
        if x_from < 0 {
            x_to -= x_from;
            x_from = 0;
        }

        // right border
        {
            let to_overflow = x_to + self.w_to - W;
            if to_overflow > 0 {
                x_to -= to_overflow;
                x_from -= to_overflow;
            }
        }
        {
            let from_overflow = x_from + self.w_from - W;
            if from_overflow > 0 {
                x_to -= from_overflow;
                x_from -= from_overflow;
            }
        }

        Optic {
            x_from,
            x_to,
            y: Cell::new(self.y.get()),
            ..*self
        }
    }

    fn get_relative(&self, mut progress: i16) -> Optic {
        let mut progress = progress % OPTIC_HEIGHT;
        Optic { x_to: (self.x_from + (self.x_to - self.x_from) * progress / (OPTIC_HEIGHT - 1)),
                w_to: (self.w_from + (self.w_to - self.w_from) * progress / (OPTIC_HEIGHT - 1)),
                y: Cell::new(self.y.get() + progress),
                ..*self
        }
    }

    fn draw(&self) {
        // draw_line(
        //     self.x_to as f32,
        //     self.y.get() as f32,
        //     self.x_from as f32,
        //     (self.y.get() as f32 + OPTIC_HEIGHT as f32).min(H as f32),
        //     6.0,
        //     RED
        // );

        // draw_line(
        //     (self.x_to + self.w_to) as f32,
        //     self.y.get() as f32,
        //     (self.x_from + self.w_from) as f32,
        //     (self.y.get() as f32 + OPTIC_HEIGHT as f32).min(H as f32),
        //     6.0,
        //     RED
        // );
        let optic_color = Color{ r: 0.9, g: 0.3, b: 0.4, a: 0.8 };

        draw_triangle(
            Vec2::new(self.x_from as f32,                 (self.y.get() + OPTIC_HEIGHT) as f32),
            Vec2::new((self.x_from + self.w_from) as f32, (self.y.get() + OPTIC_HEIGHT) as f32),
            Vec2::new((self.x_to) as f32,                 self.y.get() as f32),
            optic_color
        );

        draw_triangle(
            Vec2::new(self.x_to as f32,                   self.y.get() as f32),
            Vec2::new((self.x_to + self.w_to) as f32,     self.y.get() as f32),
            Vec2::new((self.x_from + self.w_from) as f32, (self.y.get()  + OPTIC_HEIGHT) as f32),
            optic_color
        );
    }
}

pub struct Level {
    optics: Vec<Optic>
}

impl Level {
    fn step(&self) {
        self.optics.iter().for_each(
            |optic|
            optic.step()
        );
    }

    fn shift(&mut self, shift: i16) {
        self.optics = self.optics.iter().map(|optic| optic.shift(shift)).collect();
    }
}

async fn select_level () -> Option<LevelName> {
    Some(LevelName::L1)
}

const SPEED: i16 = 8;
const W: i16 = 800;
const H: i16 = 1000;
const OPTIC_HEIGHT : i16 = 100;
const LINE_W: i16 = 10;

#[derive(Clone, Copy)]
struct Screen ([bool; W as usize]);

impl Screen {
    fn compute_pixel_unsafe(&self, opt: &Optic, i_usize: usize) -> bool {
        let i = i_usize as i16;
        let rel_w = (i - opt.x_to) as f32 / opt.w_to as f32;
        let rel_x: usize = (opt.x_from +
                            (rel_w * opt.w_from as f32) as i16) as usize;
        self.0[rel_x % W as usize]
    }

    fn compute_pixel(&self, opt: &Optic, i_usize: usize) -> bool {
        let i = i_usize as i16;
        let is_in = i >= opt.x_to && i < opt.w_to + opt.x_to;
        let is_left = i > 0 && i < opt.x_to;
        let is_right = i < W && i >= opt.w_to + opt.x_to;

        if is_in {
            self.compute_pixel_unsafe(&opt, i_usize)
        } else if is_left {
            self.compute_pixel_unsafe(&opt.get_left_complement(), i_usize)
        } else if is_right {
            self.compute_pixel_unsafe(&opt.get_right_complement(), i_usize)
        } else {
            self.0[i_usize]
        }
    }

    fn compute(&self, opt: &Optic) -> Screen {
        Screen(
            core::array::from_fn(|i_usize| self.compute_pixel(opt, i_usize))
        )
    }

    fn draw(&self, y: i16, slim: bool) {
        let mut last_flag : bool = self.0[0];
        let mut first_cell : usize = 0;

        if y > H as i16 {
            return;
        }

        self.0.iter().enumerate().for_each(
            |(i, flag)| {
                if last_flag != *flag {
                    draw_rectangle(
                        first_cell as f32,
                        if slim { y as f32 - 1.0 } else { 0.0 },
                        (i - first_cell) as f32,
                        if slim { 1.0 } else { y as f32 },
                        if *flag { ORANGE } else { BLACK }
                    );
                    first_cell = i;
                    last_flag = *flag;
                }
            }
        );

        if first_cell as i16 != W - 1 {
            draw_rectangle(
                first_cell as f32,
                if slim { y as f32 - 1.0 } else { 0.0 },
                (W - first_cell as i16) as f32,
                if slim { 1.0 } else { y as f32 },
                if last_flag { ORANGE } else { BLACK }
            );
        }
    }
}

fn mk_prism (mut prng: Xoshiro128StarStar) -> (Xoshiro128StarStar, Optic) {

    let w_from = (prng.next_u32() % (W as u32 / 3)) as i16;
    let w_to = (prng.next_u32() % (W as u32 / 3)) as i16;

    let w = max(w_from, w_to);
    let x_center = (prng.next_u32() % (W - w / 2) as u32) as i16;

    let x_from = x_center - w_from / 2;
    let x_to = x_center - w_to / 2;

    (prng, Optic { x_from, w_from, x_to, w_to, y: Cell::new(-OPTIC_HEIGHT) })
}

fn get_sec () -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut frameCounter : u64 = 0;
    let mut prng = Xoshiro128StarStar::seed_from_u64(123);

    let row_width = screen_width() / (W as f32);

    let mut screen = Screen([false; W as usize]);

    let mut s = prng.next_u32();
    for i in 0 .. W {
        if i % LINE_W == 0 {
            s = prng.next_u32();
        }
        screen.0[i as usize] = s % 2 == 1;
    }

    let mut level1 = Level {
        optics: vec![
            Optic { x_from: 10, w_from: 100, x_to: 130, w_to: 100, y: Cell::new(-100) },
            Optic { x_from: 110, w_from: 100, x_to: 10, w_to: 100, y: Cell::new(-600) },
            // Optic { x_from: 210, w_from: 100, x_to: 120, w_to: 400, y: Cell::new(-800) },
            // Optic { x_from: 110, w_from: 100, x_to: 10, w_to: 400, y: Cell::new(-600) },
            // Optic { x_from: 110, w_from: 100, x_to: 10, w_to: 400, y: Cell::new(-900) },
        ]
    };

    let mut last = get_sec();
    let mut frame_count = 0;
    let mut fps: u64 = 0;

    loop {
        level1.step();

        let mut needs_push = false;

        screen.draw(H as i16, false);

        (_, _) = level1.optics.iter().fold((screen, H), |(scr, start), optic| {
            if optic.y.get() < - OPTIC_HEIGHT {
                return (scr, start);
            }

            if optic.y.get() >= H {
                let next_scr = scr.compute(optic);
                next_scr.draw(H as i16, false);
                return (next_scr, optic.y.get());
            }

            for i in (0 .. OPTIC_HEIGHT - 1).rev() {
                let temp_optic = &optic.get_relative(OPTIC_HEIGHT - i);
                scr.compute(temp_optic)
                    .draw(optic.y.get() + i as i16, // i != 0 && i != OPTIC_HEIGHT - 1
                          true
                    )
            }

            let scr_next = scr.compute(&optic);
            scr_next.draw(optic.y.get(), false);
            optic.draw();

            (scr_next, optic.y.get())
        });

        // delete
        level1.optics = level1.optics.into_iter().filter(
            |optic|
            {
                let y = optic.y.get();
                let res = y < H as i16;
                if !res {
                    screen = screen.compute(&optic);
                    needs_push = true;
                }
                res
            }
        ).collect();

        if is_key_down(KeyCode::Right) {
            level1.shift(5);
        }

        if is_key_down(KeyCode::Left) {
            level1.shift(-5);
        }

        frameCounter += 1;

        if get_sec() == last {
            frame_count += 1;
        } else {
            fps = frame_count;
            last = get_sec();
            frame_count = 0;
        }

        draw_text(fps.to_string().as_str(), 20.0, 20.0, 30.0, RED);
        draw_text(fps.to_string().as_str(), 21.0, 21.0, 30.0, GREEN);

        if needs_push {
            let prism;
            (prng, prism) = mk_prism(prng);
            level1.optics.push(prism);
        }

        next_frame().await
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "SUN KEY".to_owned(),
        window_width: W as i32,
        window_height: H as i32,
        fullscreen: true,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optic() {
        assert_eq!(
            Optic { x_from: 10, w_from: 100, x_to: 30, w_to: 40, y: Cell::new(-20) },
            Optic { x_from: 10, w_from: 100, x_to: 30, w_to: 40, y: Cell::new(-20) }
        );
        assert_eq!(
            Optic { x_from: 10, w_from: 100, x_to: 30, w_to: 40, y: Cell::new(-20) }.get_relative(0),
            Optic { x_from: 10, w_from: 100, x_to: 10, w_to: 100, y: Cell::new(-20) },
        );
        assert_eq!(
            Optic { x_from: 10, w_from: 100, x_to: 30, w_to: 40, y: Cell::new(-20) }.get_relative(OPTIC_HEIGHT - 1),
            Optic { x_from: 10, w_from: 100, x_to: 30, w_to: 40, y: Cell::new(-20 + OPTIC_HEIGHT as i16 - 1) },
        );
        assert_eq!(
            Optic { x_from: 10, w_from: 100, x_to: 20, w_to: 50, y: Cell::new(-20) }.get_relative(5),
            Optic { x_from: 10, w_from: 100, x_to: 15, w_to: 73, y: Cell::new(-20 + OPTIC_HEIGHT as i16 / 2) },
        );
    }
}
