use std::{cell::RefCell, rc::Rc};

use winit::dpi::PhysicalSize;

pub struct Renderer {
    camera: Rc<RefCell<Camera>>,
    size: PhysicalSize<u32>,
    pixels: Vec<u32>,
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

#[derive(Default)]
pub struct Hit {
    material: u8,
    side: u8,
    x: f32,
    y: f32,
    dist: f32,
}

impl Renderer {
    pub fn new(camera: Rc<RefCell<Camera>>, size: PhysicalSize<u32>) -> Self {
        let buffer_size = size.width * size.height;
        Self {
            camera,
            size,
            pixels: vec![0; buffer_size as usize],
        }
    }

    fn write_column(&mut self, x: usize, y0: usize, y1: usize, color: u32) {
        for y in y0..y1 {
            self.pixels[y * self.size.width as usize + x] = color;
        }
    }

    fn raycast(&self, x: usize) -> Hit {
        let camera = self.camera.borrow();
        let xcam = (2. * (x as f32 / self.size.width as f32)) - 1.;
        let ray = (
            camera.facing_dir.0 + camera.view_plane.0 * xcam,
            camera.facing_dir.1 + camera.view_plane.1 * xcam,
        );

        let pos = camera.player_pos;
        let mut ipos = (pos.0 as usize, pos.1 as usize);
        let delta_dist = (ray.0.recip().abs(), ray.1.recip().abs());
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

        let mut hit = Hit::default();

        while hit.material == 0 {
            if side_dist.0 < side_dist.1 {
                side_dist.0 += delta_dist.0;
                ipos.0 = (ipos.0 as i32 + step.0) as usize;
                hit.side = 0;
            } else {
                side_dist.1 += delta_dist.1;
                ipos.1 = (ipos.1 as i32 + step.1) as usize;
                hit.side = 1;
            }

            hit.material = MAP_DATA[ipos.1 * 15 + ipos.0];
        }

        hit.x = pos.0 + side_dist.0;
        hit.y = pos.1 + side_dist.1;
        hit.dist = match hit.side {
            0 => side_dist.0 - delta_dist.0,
            _ => side_dist.1 - delta_dist.1,
        };

        hit
    }

    fn material_to_color(mat: u8, side: u8) -> u32 {
        let mut color = match mat {
            1 => 0xFF0000FF,
            2 => 0xFF00FF00,
            3 => 0xFFFF0000,
            _ => 0xFFFF00FF,
        };
        if side == 1 {
            let br = ((color & 0xFF00FF) * 0xC0) >> 8;
            let g = ((color & 0x00FF00) * 0xC0) >> 8;
            color = 0xFF000000 | (br & 0xFF00FF) | (g & 0x00FF00);
        }
        color
    }

    pub fn render(&mut self) {
        let (width, height) = (self.size.width as usize, self.size.height as usize);
        for x in 0..width {
            let hit = self.raycast(x);

            let color = Self::material_to_color(hit.material, hit.side);

            let h = (height as f32 / hit.dist) as usize;
            let y0 = usize::max((height / 2) - (h / 2), 0);
            let y1 = usize::min((height / 2) + (h / 2), height - 1);

            self.write_column(x, 0, y0, 0xFF202020);
            self.write_column(x, y0, y1, color);
            self.write_column(x, y1, height, 0xFF404040);
        }
    }

    pub fn pixels(&self) -> &[u8] {
        bytemuck::cast_slice::<u32, u8>(&self.pixels)
    }
}
