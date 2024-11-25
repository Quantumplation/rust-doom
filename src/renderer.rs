use std::sync::Arc;

use wgpu::{Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue};

pub struct Renderer {
    screen: Arc<wgpu::Texture>,
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

impl Renderer {
    pub fn new(screen: Arc<wgpu::Texture>) -> Self {
        let size = (screen.width() * screen.height() * 4) as usize;
        Self {
            screen,
            pixels: vec![0; size],
        }
    }

    pub fn render(
        &mut self,
        player_pos: (f32, f32),
        facing_dir: (f32, f32),
        view_plane: (f32, f32),
    ) {
        for x in 0..800 {
            let xcam = (2. * (x as f32 / 800.)) - 1.;
            let ray = (
                facing_dir.0 + view_plane.0 * xcam,
                facing_dir.1 + view_plane.1 * xcam,
            );
            let pos = player_pos;
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

            let h = (600. / dperp) as u32;
            let y0 = u32::max(300 - (h / 2), 0) as usize;
            let y1 = u32::min(300 + (h / 2), 600 - 1) as usize;

            for y in 0..y0 {
                self.pixels[(y * 800 + x) * 4 + 3] = 0xFF;
                self.pixels[(y * 800 + x) * 4 + 2] = 0x20;
                self.pixels[(y * 800 + x) * 4 + 1] = 0x20;
                self.pixels[(y * 800 + x) * 4 + 0] = 0x20;
            }
            for y in y0..=y1 {
                self.pixels[(y * 800 + x) * 4 + 3] = ((color & 0xFF000000) >> 24) as u8;
                self.pixels[(y * 800 + x) * 4 + 2] = ((color & 0x00FF0000) >> 16) as u8;
                self.pixels[(y * 800 + x) * 4 + 1] = ((color & 0x0000FF00) >> 8) as u8;
                self.pixels[(y * 800 + x) * 4 + 0] = (color & 0x000000FF) as u8;
            }
            for y in y1..600 {
                self.pixels[(y * 800 + x) * 4 + 3] = 0xFF;
                self.pixels[(y * 800 + x) * 4 + 2] = 0x40;
                self.pixels[(y * 800 + x) * 4 + 1] = 0x40;
                self.pixels[(y * 800 + x) * 4 + 0] = 0x40;
            }
        }
    }

    pub fn queue(&self, queue: &Queue) {
        let (width, height) = (self.screen.width(), self.screen.height());
        let texture = ImageCopyTexture {
            texture: &self.screen,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        };
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let data_layout = ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(800 * 4),
            rows_per_image: Some(600),
        };
        queue.write_texture(texture, &self.pixels, data_layout, size);
    }
}
