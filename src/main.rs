#![feature(cell_update)]

use macroquad::prelude::*;
use macroquad::color::Color;
use std::cell::Cell;
use rand_xoshiro::Xoshiro128StarStar;
use rand_core::SeedableRng;
use rand_core::RngCore;

struct Game {

}

pub enum LevelName {
    L1 = 0,
}

#[derive(PartialEq,Eq,Debug)]
pub struct Optic {
    x_from: u16,
    w_from: u16,
    x_to: u16,
    w_to: u16,
    y: Cell<i16>
}

impl Optic {
    fn step(&self) {
        self.y.update(|y| y + 1);
    }

    fn get_relative(&self, progress_u: u16) -> Optic {
        let progress = (progress_u as i16 % OPTIC_HEIGHT as i16);
        Optic { x_from: self.x_from,
                w_from: self.w_from,
                x_to:
                  ( self.x_from as i16 + (self.x_to as i16 - self.x_from as i16) * progress / (OPTIC_HEIGHT - 1) as i16) as u16,
                w_to: (self.w_from as i16 + (self.w_to as i16 - self.w_from as i16) * progress / (OPTIC_HEIGHT - 1) as i16) as u16,
                y: Cell::new(self.y.get() + progress_u as i16)
        }
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

    fn render(&self) {
        self.optics.iter().for_each(
            |optic|
            draw_rectangle(
                0.0,
                0.0,
                100.0,
                Cell::get(&optic.y) as f32,
                GREEN
            )
        );
    }
}

async fn select_level () -> Option<LevelName> {
    Some(LevelName::L1)
}

const W: u16 = 800;
const H: u16 = 1000;

const OPTIC_HEIGHT : u16 = 30;

#[derive(Clone)]
pub struct Screen {
    area: [bool; W as usize]
}

impl Screen {
    fn compute(&self, opt: &Optic) -> Screen {
        Screen{
            area:
            core::array::from_fn(
                |i_usize|
                {
                    let i = i_usize as u16;
                    let is_in = i >= opt.x_to && i < opt.w_to + opt.x_to;

                    if is_in {
                        let rel_w = (i - opt.x_to) as f32 / opt.w_to as f32;
                        let rel_x: usize = (opt.x_from +
                                            (rel_w * opt.w_from as f32) as u16) as usize;
                        (*self).area[rel_x]
                    } else { (*self).area[i_usize] }
                }
            )
        }
    }

    fn draw(&self, y: i16) {
        self.area.iter().enumerate().for_each(
            |(i, flag)|
            draw_rectangle(
                i as f32,
                0.0,
                1.0,
                y as f32,
                if *flag { ORANGE } else { BLACK })
        );
    }
}

#[macroquad::main("SHAPES")]
async fn main() {
    let mut frameCounter : u64 = 0;
    let mut prng = Xoshiro128StarStar::seed_from_u64(123);

    let row_width = screen_width() / (W as f32);

    let mut screen = Screen { area: [false; W as usize] };

    let LINE_W = 5;

    let mut s = prng.next_u32();
    for i in 0 .. W {
        if i % LINE_W == 0 {
            s = prng.next_u32();
        }
        screen.area[i as usize] = s % 2 == 1;
    }

    let mut level1 = Level {
        optics: vec![
            Optic { x_from: 10, w_from: 100, x_to: 30, w_to: 40, y: Cell::new(-20) },
            Optic { x_from: 10, w_from: 100, x_to: 30, w_to: 80, y: Cell::new(-200) },
        ]
    };

    loop {
        let mut acc_screen = screen.clone();

        clear_background(Color{r:200.,g:200.,b:200.,a:0.});

        screen.draw(H as i16);

        level1.step();
        level1.render();

        level1.optics.iter().for_each(
            |optic|
            {
                let mut temp_screen = acc_screen.clone(); //TODO remove clone
                for i in 0 .. OPTIC_HEIGHT - 1 {
                    let temp_optic = &optic.get_relative(OPTIC_HEIGHT - i);
                    acc_screen.compute(temp_optic)
                        .draw(temp_optic.y.get() as i16 - i as i16)
                }

                acc_screen = acc_screen.compute(&optic);
                acc_screen.draw(optic.y.get() - OPTIC_HEIGHT as i16)
                // temp_screen.draw(optic.y.get())
            }
        );

        // draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        // draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        // draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        draw_text("IT WORKS!", 20.0 + frameCounter as f32, 20.0, 30.0, DARKGRAY);
        frameCounter += 1;
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
