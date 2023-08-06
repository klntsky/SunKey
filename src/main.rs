use macroquad::color::Color;
use macroquad::input::*;
use macroquad::prelude::*;
use macroquad::models::*;
use macroquad::time::{get_fps};
use rand_core::RngCore;
use rand_core::SeedableRng;
use rand_xoshiro::Xoshiro128StarStar;
use std::cell::Cell;
use std::cmp::*;
use std::f32::consts::{PI};
use std::time::{SystemTime, UNIX_EPOCH};
use macroquad::texture::*;

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
        self.y.set(self.y.get() + SPEED);
    }

    /// Shift the optic left or right
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

    fn get_relative(&self, progress: i16, height: i16) -> Optic {
        let progress = progress % height;
        Optic {
            x_to: self.x_from + (self.x_to - self.x_from) * progress / (height - 1),
            w_to: self.w_from + (self.w_to - self.w_from) * progress / (height - 1),
            y: Cell::new(self.y.get() + progress),
            ..*self
        }
    }

    fn draw(&self, screen_scale: &ScreenScale) {
        let optic_color = Color { r: 0.9, g: 0.3, b: 0.4, a: 0.8 };

        draw_triangle_rel(
            Vec2::new(self.x_from as f32,                 (self.y.get() + OPTIC_HEIGHT) as f32),
            Vec2::new((self.x_from + self.w_from) as f32, (self.y.get() + OPTIC_HEIGHT) as f32),
            Vec2::new((self.x_to) as f32,                 self.y.get() as f32),
            optic_color,
            &screen_scale
        );

        draw_triangle_rel(
            Vec2::new(self.x_to as f32,                   self.y.get() as f32),
            Vec2::new((self.x_to + self.w_to) as f32,     self.y.get() as f32),
            Vec2::new((self.x_from + self.w_from) as f32, (self.y.get()  + OPTIC_HEIGHT) as f32),
            optic_color,
            &screen_scale
        );
    }
}

pub struct ScreenScale {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl ScreenScale {
    fn new() -> ScreenScale {
        ScreenScale {
            x: W as f32 / screen_width(),
            y: H as f32 / screen_height(),
            w: screen_width(),
            h: screen_height()
        }
    }

    fn ratio (&self) -> f32 {
        (W as f32 / self.w).max(
            H as f32 / self.h
        )
    }

    fn x(&self, x: f32) -> f32 {
        // TODO: restore shift when performance is fixed
        let shift = 0.0; // W as f32 / self.ratio() - self.w as f32;
        return x as f32 / self.ratio() - shift / 2.0;
    }

    fn y(&self, y: f32) -> f32 {
        // TODO: restore shift when performance is fixed
        let shift = 0.0; // H as f32 / self.ratio() - self.h as f32;
        return y as f32 / self.ratio() - shift / 2.0;
    }
}

pub struct Level {
    optics: Vec<Optic>
}

impl Level {
    fn step(&self) {
        self.optics.iter().for_each(|optic| optic.step());
    }

    fn shift(&mut self, shift: i16) {
        self.optics = self.optics.iter().map(
            |optic| optic.shift(shift)).collect();
    }
}

const SPEED: i16 = 8;
const H_SPEED: i16 = 8;
const W: i16 = 720;
const H: i16 = 1600;
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
        if rel_x >= W as usize {
            false
        } else {
            self.0[rel_x % W as usize]
        }
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

    fn draw(&self, y: i16, ss: &ScreenScale, slim: bool) {
        let mut last_flag : bool = self.0[0];
        let mut first_cell : usize = 0;

        if y > H as i16 {
            return;
        }

        self.0.iter().enumerate().for_each(
            |(i, flag)| {
                if last_flag != *flag {
                    draw_rectangle_rel(
                        first_cell as f32,
                        if slim { y as f32 - 1.0 } else { 0.0 },
                        (i - first_cell) as f32,
                        if slim { 1.0 } else { y as f32 },
                        if *flag { ORANGE } else { BLACK },
                        ss
                    );
                    first_cell = i;
                }
                last_flag = *flag;
            }
        );

        draw_rectangle_rel(
            first_cell as f32,
            if slim { y as f32 } else { 0.0 },
            (W - first_cell as i16) as f32,
            if slim { 1.0 } else { y as f32 },
            if !last_flag { ORANGE } else { BLACK },
            ss
        );
    }
}

fn mk_prism (prng: &mut Xoshiro128StarStar) -> Optic {
    let w_from = (prng.next_u32() % (W as u32 / 3)) as i16;
    let w_to = (prng.next_u32() % (W as u32 / 3)) as i16;

    let w = max(w_from, w_to);
    let x_center = (prng.next_u32() % (W - w / 2) as u32) as i16;

    let x_from = x_center - w_from / 2;
    let x_to = x_center - w_to / 2;

    Optic { x_from, w_from, x_to, w_to, y: Cell::new(-OPTIC_HEIGHT) }
}

fn get_sec () -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}

fn draw_rectangle_rel (x: f32, y: f32, w: f32, h: f32, color: Color, ss: &ScreenScale) {
    draw_rectangle(ss.x(x), ss.y(y), ss.x(w), ss.y(h), color);
}

fn draw_triangle_rel (a: Vec2, b: Vec2, c: Vec2, color: Color, ss: &ScreenScale) {
    draw_triangle(
        Vec2::new(ss.x(a.x), ss.y(a.y)),
        Vec2::new(ss.x(b.x), ss.y(b.y)),
        Vec2::new(ss.x(c.x), ss.y(c.y)),
        color
    );
}

async fn draw_sun (ray_count: i16, screen_scale: &ScreenScale) {
    let mut ray_progress = 0;
    let mut progress = 0.05;
    let pi2 = PI*2.0;

    // 1.1 because we want a little delay
    while progress < 1.1 {
        for i in 0..ray_count {
            let base = ((pi2/ray_count as f32) * i as f32) + progress;
            let from = base;
            let to = ((pi2/ray_count as f32) * (i as f32 + progress)) + progress;

            draw_triangle_rel(
                Vec2::new(
                    (W/2) as f32,
                    (H/2) as f32
                ),
                Vec2::new(
                    (W/2) as f32 + from.sin() * 2.0 * W as f32,
                    (H/2) as f32 + from.cos() * 2.0 * H as f32
                ),
                Vec2::new(
                    (W/2) as f32 + to.sin() * 2.0 * W as f32,
                    (H/2) as f32 + to.cos() * 2.0 * H as f32
                ),
                ORANGE,
                &screen_scale
            );
        }

        progress += 0.015;
        next_frame().await
    }
}

fn generate_screen (prng : &mut Xoshiro128StarStar) -> Screen {
    let mut screen = Screen([false; W as usize]);

    let mut s = prng.next_u32();
    for i in 0 .. W {
        if i % LINE_W == 0 {
            s = prng.next_u32();
        }
        screen.0[i as usize] = s % 2 == 1;
    }

    screen
}

async fn transition_to_screen (
    screen_target: &Screen,
    screen_scale: &ScreenScale,
    prng: &mut Xoshiro128StarStar
) {
    let mut screen = Screen([false; W as usize]);
    let mut all = false;

    while !all {
        all = true;
        for (i, flag) in screen_target.0.iter().enumerate() {
            if screen.0[i] != *flag {
                if prng.next_u32() % 10 == 0 {
                    screen.0[i] = *flag;
                } else {
                    all = false;
                }
            }
        }
        screen.draw(H, &screen_scale, false);
        next_frame().await;
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut prng = Xoshiro128StarStar::seed_from_u64(123);

    let mut screen = generate_screen(&mut prng);

    let mut level1 = Level {
        optics: vec![
            Optic { x_from: 10, w_from: 100, x_to: 130, w_to: 100, y: Cell::new(-100) },
            Optic { x_from: 110, w_from: 100, x_to: 10, w_to: 100, y: Cell::new(-600) },
            Optic { x_from: 10, w_from: 100, x_to: 130, w_to: 100, y: Cell::new(-1200) },
            // Optic { x_from: 210, w_from: 100, x_to: 120, w_to: 400, y: Cell::new(-800) },
            // Optic { x_from: 110, w_from: 100, x_to: 10, w_to: 400, y: Cell::new(-600) },
            // Optic { x_from: 110, w_from: 100, x_to: 10, w_to: 400, y: Cell::new(-900) },
        ]
    };

    let mut frame_count = 0;
    let mut last_touch = None;
    let mut screen_scale = ScreenScale::new();
    let mut last_fps = 0;

    // fight the screen size glitch
    set_fullscreen(false);
    next_frame().await;
    set_fullscreen(true);
    next_frame().await;
    set_fullscreen(false);
    next_frame().await;
    request_new_screen_size(screen_width(), screen_height());
    next_frame().await;

    screen_scale = ScreenScale::new();

    draw_sun(5, &screen_scale).await;

    transition_to_screen(&screen, &screen_scale, &mut prng).await;
    let mut meshes : Vec<Mesh> = vec![];

    loop {
        screen_scale = ScreenScale::new();

        level1.step();

        let mut needs_push = false;

        screen.draw(H as i16, &screen_scale, false);

        _ = level1.optics.iter().fold((screen, H, 0), |(scr, start, ix), optic| {
            // Optic is too high - skip
            if optic.y.get() < - OPTIC_HEIGHT {
                return (scr, start, ix + 1);
            }

            // Optic is too far below
            if optic.y.get() >= H {
                let next_scr = scr.compute(optic);
                next_scr.draw(H as i16, &screen_scale, false);
                return (next_scr, optic.y.get(), ix + 1);
            }

            let mut arr =
                &mut [0 as u8; W as usize * OPTIC_HEIGHT as usize * 4];

            // match meshes.get(ix) {
            //     // Generate mesh
            //     None => {
                    for y in (0 .. OPTIC_HEIGHT) {
                        let temp_optic = &optic.get_relative(OPTIC_HEIGHT - y, OPTIC_HEIGHT);
                        let temp_scr = scr.compute(temp_optic);

                        temp_scr.0.iter().enumerate().for_each(|(x, flag)| {
                            let shift : usize = (y as usize * W as usize + x) * 4;
                            if !*flag {
                                arr[shift] = 255;
                                arr[shift + 1] = 161;
                                arr[shift + 2] = 0;
                                arr[shift + 3] = 255;
                            } else {
                                arr[shift] = 0;
                                arr[shift + 1] = 0;
                                arr[shift + 2] = 0;
                                arr[shift + 3] = 255;
                            }
                        });
                    }

                    let texture = Texture2D::from_rgba8(
                        W as u16,
                        OPTIC_HEIGHT as u16,
                        arr
                    );

                    let y_from = screen_scale.y(optic.y.get() as f32).floor() - 2.0; // TODO: why -2.0?
                    let y_to = screen_scale.y(optic.y.get() as f32 + OPTIC_HEIGHT as f32);
                    let x_from = screen_scale.x(0.0);
                    let x_to = screen_scale.x(W as f32);

                    let mesh = Mesh {
                        vertices: vec![
                            macroquad::models::Vertex{
                                position: Vec3 { x: x_from, y: y_from, z: 0.0 },
                                uv: Vec2 { x: 0.0, y: 0.0 },
                                color: WHITE
                            },
                            macroquad::models::Vertex{
                                position: Vec3{ x: x_from, y: y_to, z: 0.0 },
                                uv: Vec2 { x: 0.0, y: 1.0 },
                                color: WHITE
                            },
                            macroquad::models::Vertex{
                                position: Vec3 { x: x_to, y: y_to, z: 0.0 },
                                uv: Vec2 { x: 1.0, y: 1.0 },
                                color: WHITE
                            },
                            macroquad::models::Vertex{
                                position: Vec3 { x: x_to, y: y_from, z: 0.0 },
                                uv: Vec2 { x: 1.0, y: 0.0 },
                                color: WHITE
                            },
                        ],
                        indices: vec![0,1,2,2,3,0],
                        texture: Some(texture.clone())
                    };
                    draw_mesh(&Mesh {
                        vertices: vec![
                            macroquad::models::Vertex{
                                position: Vec3 { x: x_from, y: y_from, z: 0.0 },
                                uv: Vec2 { x: 0.0, y: 0.0 },
                                color: WHITE
                            },
                            macroquad::models::Vertex{
                                position: Vec3{ x: x_from, y: y_to, z: 0.0 },
                                uv: Vec2 { x: 0.0, y: 1.0 },
                                color: WHITE
                            },
                            macroquad::models::Vertex{
                                position: Vec3 { x: x_to, y: y_to, z: 0.0 },
                                uv: Vec2 { x: 1.0, y: 1.0 },
                                color: WHITE
                            },
                            macroquad::models::Vertex{
                                position: Vec3 { x: x_to, y: y_from, z: 0.0 },
                                uv: Vec2 { x: 1.0, y: 0.0 },
                                color: WHITE
                            },
                        ],
                        indices: vec![0,1,2,2,3,0],
                        texture: Some(texture)
                    });
                    meshes.push(mesh);
            //         // &meshes[meshes.len()-1]
            //     }
            //     Some(mesh) => draw_mesh(mesh)
            // };

            // draw_mesh(mesh);

            // for i in (0 .. OPTIC_HEIGHT - 1).rev() {
            //     let temp_optic = &optic.get_relative(OPTIC_HEIGHT - i, OPTIC_HEIGHT);
            //     scr.compute(temp_optic)
            //         .draw(
            //             optic.y.get() + i as i16,
            //             &screen_scale,
            //             true
            //         )
            // }

            let scr_next = scr.compute(&optic);
            scr_next.draw(optic.y.get(), &screen_scale, false);
            optic.draw(&screen_scale);

            (scr_next, optic.y.get(), ix + 1)
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

        // Handle <- and ->
        {
            if is_key_down(KeyCode::Right) {
                level1.shift(H_SPEED);
            }

            if is_key_down(KeyCode::Left) {
                level1.shift(-H_SPEED);
            }
        }

        // Handle touches
        {
            let touche_vec = touches();
            touche_vec.get(0).iter().for_each(|touch| {
                match touch.phase {
                    TouchPhase::Started =>
                        last_touch = Some(touch.position.x),
                    TouchPhase::Moved => {
                        match last_touch {
                            Some(old_pos) => {
                                level1.shift((touch.position.x - old_pos) as i16);
                                last_touch = Some(touch.position.x);
                            },
                            _ => ()
                        }
                    },
                    TouchPhase::Ended =>
                        last_touch = None,
                    _ => ()
                }
            });
        }

        frame_count += 1;

        if frame_count % 30 == 0 {
            last_fps = get_fps();
        }
        draw_text(last_fps.to_string().as_str(), 20.0, 20.0, 30.0, RED);

        if needs_push {
            level1.optics.push(mk_prism(&mut prng));
        }

        next_frame().await
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "SUN KEY".to_owned(),
        window_width: W as i32,
        window_height: H as i32,
        high_dpi: true,
        fullscreen: false,
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
            Optic { x_from: 10, w_from: 100, x_to: 30, w_to: 40, y: Cell::new(-20) }.get_relative(0, 10),
            Optic { x_from: 10, w_from: 100, x_to: 10, w_to: 100, y: Cell::new(-20) },
        );
        assert_eq!(
            Optic { x_from: 10, w_from: 100, x_to: 30, w_to: 40, y: Cell::new(-20) }.get_relative(10 - 1, 10),
            Optic { x_from: 10, w_from: 100, x_to: 30, w_to: 40, y: Cell::new(-20 + 10 as i16 - 1) },
        );
        assert_eq!(
            Optic { x_from: 10, w_from: 100, x_to: 20, w_to: 50, y: Cell::new(-20) }.get_relative(5, 10),
            Optic { x_from: 10, w_from: 100, x_to: 15, w_to: 73, y: Cell::new(-20 + 10 as i16 / 2) },
        );
    }
}
