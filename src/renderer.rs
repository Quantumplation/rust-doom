use std::sync::{Arc, Mutex};

use winit::dpi::PhysicalSize;

pub struct Renderer {
    camera: Arc<Mutex<Camera>>,
    size: PhysicalSize<u32>,
    pixels: Vec<u8>,
}

#[rustfmt::skip]
const MAP_DATA: [u8; 15*15] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 2, 0, 0, 3, 3, 3, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

pub struct Camera {
    pub player_pos: (f32, f32),
    pub facing_dir: (f32, f32),
    pub view_plane: (f32, f32),
}

impl Renderer {
    pub fn new(camera: Arc<Mutex<Camera>>, size: PhysicalSize<u32>) -> Self {
        let buffer_size = size.width * size.height * 4;
        Self {
            camera,
            size,
            pixels: vec![0; buffer_size as usize],
        }
    }

    pub fn render(&mut self) {
        let camera = self.camera.lock().unwrap();
        let (width, height) = (self.size.width as usize, self.size.height as usize);
        for x in 0..width {
            let xcam = (2. * (x as f32 / width as f32)) - 1.;
            let ray = (
                camera.facing_dir.0 + camera.view_plane.0 * xcam,
                camera.facing_dir.1 + camera.view_plane.1 * xcam,
            );
            let pos = camera.player_pos;
            let mut ipos = (pos.0 as usize, pos.1 as usize);
            let delta_dist = (
                if ray.0.abs() < 1e-20 {
                    1e30
                } else {
                    ray.0.recip().abs()
                },
                if ray.1.abs() < 1e-20 {
                    1e30
                } else {
                    ray.1.recip().abs()
                },
            );
            let mut side_dist = (
                if ray.0.abs() < 1e-20 {
                    pos.0 - ipos.0 as f32
                } else {
                    ipos.0 as f32 + 1. - pos.0
                },
                if ray.1.abs() < 1e-20 {
                    pos.1 - ipos.1 as f32
                } else {
                    ipos.1 as f32 + 1. - pos.1
                },
            );

            let step = (ray.0.signum() as i32, ray.1.signum() as i32);

            let mut hit = (0, 0, (0., 0.));

            while hit.0 == 0 {
                if side_dist.0 < side_dist.1 {
                    side_dist.0 += delta_dist.0;
                    ipos.0 = (ipos.0 as i32 + step.0) as usize;
                    hit.1 = 0;
                } else {
                    side_dist.1 += delta_dist.1;
                    ipos.1 = (ipos.1 as i32 + step.1) as usize;
                    hit.1 = 1;
                }

                hit.0 = MAP_DATA[ipos.1 * 15 + ipos.0];
            }

            let mut color: u32 = match hit.0 {
                1 => 0xFF0000FF,
                2 => 0xFF00FF00,
                3 => 0xFFFF0000,
                _ => 0xFFFF00FF,
            };
            if hit.1 == 1 {
                let br = ((color & 0xFF00FF) * 0xC0) >> 8;
                let g = ((color & 0x00FF00) * 0xC0) >> 8;
                color = 0xFF000000 | (br & 0xFF00FF) | (g & 0x00FF00);
            }
            hit.2 = (pos.0 + side_dist.0, pos.1 + side_dist.1);

            let dperp = match hit.1 {
                0 => side_dist.0 - delta_dist.0,
                _ => side_dist.1 - delta_dist.1,
            };

            let h = (height as f32 / dperp) as usize;
            let y0 = usize::max((height / 2) - (h / 2), 0);
            let y1 = usize::min((height / 2) + (h / 2), height - 1);

            for y in 0..y0 {
                self.pixels[(y * width + x) * 4 + 3] = 0xFF;
                self.pixels[(y * width + x) * 4 + 2] = 0x20;
                self.pixels[(y * width + x) * 4 + 1] = 0x20;
                self.pixels[(y * width + x) * 4 + 0] = 0x20;
            }
            for y in y0..=y1 {
                self.pixels[(y * width + x) * 4 + 3] = ((color & 0xFF000000) >> 24) as u8;
                self.pixels[(y * width + x) * 4 + 2] = ((color & 0x00FF0000) >> 16) as u8;
                self.pixels[(y * width + x) * 4 + 1] = ((color & 0x0000FF00) >> 8) as u8;
                self.pixels[(y * width + x) * 4 + 0] = (color & 0x000000FF) as u8;
            }
            for y in y1..height {
                self.pixels[(y * width + x) * 4 + 3] = 0xFF;
                self.pixels[(y * width + x) * 4 + 2] = 0x40;
                self.pixels[(y * width + x) * 4 + 1] = 0x40;
                self.pixels[(y * width + x) * 4 + 0] = 0x40;
            }
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}
