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
    fn step(&self) {
        self.y.update(|y| y + SPEED);
    }

    fn shift(&self, shift: i16) -> Optic {
        Optic {
            x_from: self.x_from + shift,
            x_to: self.x_to + shift,
            w_from: self.w_from,
            w_to: self.w_to,
            y: Cell::new(self.y.get())
        }
    }

    fn get_relative(&self, progress_u: u16) -> Optic {
        let progress = progress_u as i16 % OPTIC_HEIGHT as i16;
        Optic { x_from: self.x_from,
                w_from: self.w_from, // TODO move progress() to utils
                x_to: (self.x_from + (self.x_to - self.x_from) * progress / (OPTIC_HEIGHT - 1) as i16),
                w_to: (self.w_from + (self.w_to - self.w_from) * progress / (OPTIC_HEIGHT - 1) as i16),
                y: Cell::new(self.y.get() + progress_u as i16)
        }
    }

    fn draw(&self) {
        draw_line(
            self.x_to as f32,
            self.y.get() as f32,
            self.x_from as f32,
            (self.y.get() as f32 + OPTIC_HEIGHT as f32).min(H as f32),
            3.0,
            RED
        );

        draw_line(
            (self.x_to + self.w_to) as f32,
            self.y.get() as f32,
            (self.x_from + self.w_from) as f32,
            (self.y.get() as f32 + OPTIC_HEIGHT as f32).min(H as f32),
            3.0,
            RED
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

const SPEED: i16 = 1;
const W: i16 = 800;
const H: i16 = 1000;
const OPTIC_HEIGHT : u16 = 30;
const LINE_W: i16 = 10;

#[derive(Clone, Copy)]
struct Screen ([bool; W as usize]);

impl Screen {
    fn compute(&self, opt: &Optic) -> Screen {
        Screen(
            core::array::from_fn(
                |i_usize|
                {
                    let i = i_usize as i16;
                    let is_in = i >= opt.x_to && i < opt.w_to + opt.x_to;

                    if is_in {
                        let rel_w = (i - opt.x_to) as f32 / opt.w_to as f32;
                        let rel_x: usize = (opt.x_from +
                                            (rel_w * opt.w_from as f32) as i16) as usize;
                        self.0[rel_x % W as usize]
                    } else { self.0[i_usize] }
                }
            )
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
                // if i == W as usize - 1 {
                //     draw_rectangle(
                //         first_cell as f32,
                //         if slim { y as f32 - 1.0 } else { 0.0 },
                //         (i - first_cell) as f32,
                //         if slim { 1.0 } else { y as f32 },
                //         if *flag { ORANGE } else { BLACK }
                //     );
                // }
            }
        );
    }
}
fn get_sec () -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}

#[macroquad::main("SHAPES")]
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
            Optic { x_from: 10, w_from: 400, x_to: 130, w_to: 100, y: Cell::new(-20) },
            Optic { x_from: 210, w_from: 100, x_to: 120, w_to: 400, y: Cell::new(-800) },
            Optic { x_from: 110, w_from: 100, x_to: 10, w_to: 400, y: Cell::new(-400) },
            Optic { x_from: 110, w_from: 100, x_to: 10, w_to: 400, y: Cell::new(-600) },
            // Optic { x_from: 110, w_from: 100, x_to: 10, w_to: 400, y: Cell::new(-900) },
        ]
    };

    let mut last = get_sec();
    let mut frame_count = 0;
    let mut fps: u64 = 0;

    loop {
        // clear_background(Color{r:200.,g:200.,b:200.,a:0.});
        level1.step();

        let mut needs_push = false;
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

        if needs_push {
            let w_from = (prng.next_u32() % (W as u32 / 2)) as i16;
            let x_from = W / 2 - w_from / 2;
            let w_to = (prng.next_u32() % (W as u32 / 2)) as i16;
            let x_to = W / 2 - w_to / 2;
            level1.optics.push(
                Optic { x_from, w_from, x_to, w_to, y: Cell::new(-20) }
            );
        }

        let mut acc_screen = screen.clone();

        screen.draw(H as i16, false);

        level1.optics.iter().for_each(
            |optic|
            {
                if optic.y.get() < - (OPTIC_HEIGHT as i16) {
                    return;
                }

                let mut temp_screen = acc_screen;
                for i in (0 .. OPTIC_HEIGHT - 1).rev() {
                    let temp_optic = &optic.get_relative(OPTIC_HEIGHT - i);
                    acc_screen.compute(temp_optic)
                        .draw(optic.y.get() + i as i16, // i != 0 && i != OPTIC_HEIGHT - 1
                              true
                        )
                }

                acc_screen = acc_screen.compute(&optic);
                acc_screen.draw(optic.y.get(), false);
                optic.draw();
            }
        );

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
        next_frame().await
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
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
