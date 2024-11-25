use std::{cell::RefCell, rc::Rc};

use cgmath::{ElementWise, Vector2, Zero};
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
    pub player_pos: Vector2<f32>,
    pub facing_dir: Vector2<f32>,
    pub view_plane: Vector2<f32>,
}

pub struct Hit {
    material: u8,
    side: u8,
    point: Vector2<f32>,
    dist: f32,
}
impl Default for Hit {
    fn default() -> Self {
        Hit {
            material: 0,
            side: 0,
            point: Vector2::zero(),
            dist: 0.,
        }
    }
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
        let ray = Vector2::new(
            camera.facing_dir.x + camera.view_plane.x * xcam,
            camera.facing_dir.y + camera.view_plane.y * xcam,
        );

        let pos = camera.player_pos;
        let mut ipos = Vector2::new(pos.x as usize, pos.y as usize);
        let delta_dist = Vector2::new(ray.x.recip().abs(), ray.y.recip().abs());
        let mut side_dist = Vector2::new(
            if ray.x.abs() < 1e-20 {
                pos.x - ipos.x as f32
            } else {
                ipos.x as f32 + 1. - pos.x
            },
            if ray.y.abs() < 1e-20 {
                pos.y - ipos.y as f32
            } else {
                ipos.y as f32 + 1. - pos.y
            },
        );

        let step = Vector2::new(ray.x.signum() as i32, ray.y.signum() as i32);

        let mut hit = Hit::default();

        while hit.material == 0 {
            if side_dist.x < side_dist.y {
                side_dist.x += delta_dist.x;
                ipos.x = (ipos.x as i32 + step.x) as usize;
                hit.side = 0;
            } else {
                side_dist.y += delta_dist.y;
                ipos.y = (ipos.y as i32 + step.y) as usize;
                hit.side = 1;
            }

            hit.material = MAP_DATA[ipos.y * 15 + ipos.x];
        }

        hit.point = pos.add_element_wise(side_dist);
        hit.dist = match hit.side {
            0 => side_dist.x - delta_dist.x,
            _ => side_dist.y - delta_dist.y,
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
