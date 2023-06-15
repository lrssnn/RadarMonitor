use glium::texture::RawImage2d;
use glium::texture::Texture2d;
use std::fs;

use super::DL_DIR;

pub enum RenderableType {
    MainImage,
    BottomSlice,
}

fn get_type_matrix(t: RenderableType) -> [[f32; 4]; 4] {
    let ((sx, sy), (tx, ty)) = match t {
        // Window height is 640, main image height is 512, so vertical scale is 80%
        // Window centre point is 320px from top, image centre point needs to go 256px from top
        // so vertical offset is 64 pixels which is 20% (0.2) of the 640 window height
        RenderableType::MainImage => ((1.0, 0.80), (0.0, 0.2)),
        // vertical offset is 256 pixels which is 80% (0.8) of the 640 window height
        // 1 percent of the horizontal width
        RenderableType::BottomSlice => ((0.01, 0.20), (-1.0, -0.8)),
    };

    [
        [sx,  0.0, 0.0, 0.0],
        [0.0, sy,  0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [tx,  ty,  0.0, 1.0]
    ]
}

pub struct Renderable {
    pub matrix: [[f32; 4]; 4],      // Transformation Matrix

    pub img: String,                // Filename for image texture
    pub texture: Option<Texture2d>, // Lazy loaded texture object from above
}

impl Renderable {
    pub fn from_disk_image(img: &str, renderable_type: RenderableType) -> Self {
        Renderable {
            matrix: get_type_matrix(renderable_type),
            img: img.to_owned(),
            texture: None,
        }
    }

    pub fn from_location_folder(lc_code: &str) -> Vec<Self> {
        let dir = &(DL_DIR.to_string() + lc_code + "/");
        let files = fs::read_dir(dir).expect("Error reading image directory");

        let mut file_names: Vec<_> = files
            .map(|e| {
                e.expect("Error reading image filename")
                    .file_name()
                    .into_string()
                    .expect("Error extracting image filename")
            })
            .collect();

        file_names.sort();

        file_names
            .iter()
            .map(|e| {
                let mut new_name = e.clone();
                new_name.remove(0);
                let r = Renderable::from_disk_image(&(dir.to_string() + &new_name), RenderableType::MainImage);
                fs::rename(dir.to_string() + e, dir.to_string() + &new_name)
                    .expect("Error renaming file");
                r
            })
            .collect()
    }

    pub fn get_texture(&mut self, display: &glium::Display) -> &Texture2d {
        // We lazy load the textures, so the first time we ask for this we need to populate this
        if self.texture.is_none() {
            let image = image::open(&self.img)
                .expect("Error opening image file")
                .to_rgba8();

            let image_dim = image.dimensions();
            let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dim);
            let texture =
                Texture2d::new(display, image).expect("Error creating texture from image");
            self.texture = Some(texture);
        }

        if let Some(tex) = &self.texture {
            tex
        } else {
            panic!("Texture lazy load is broken");
        }
    }
}
