use glium::texture::RawImage2d;
use glium::texture::Texture2d;
use std::fs;

use super::DL_DIR;

pub struct Renderable {
    pub translation: (f32, f32), // offset of the centre of the image, where the centre of the window is (0, 0)
    pub scale: f32,              // scale where 1.0 is the entire window

    pub img: String,                // Filename for image texture
    pub texture: Option<Texture2d>, // Lazy loaded texture object from above
}

impl Renderable {
    pub fn from_disk_image(img: &str, translation: (f32, f32)) -> Self {
        Renderable {
            translation,
            scale: 0.5,
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
                let r = Renderable::from_disk_image(&(dir.to_string() + &new_name), (0.5, 0.5));
                fs::rename(&(dir.to_string() + e), &(dir.to_string() + &new_name))
                    .expect("Error renaming file");
                r
            })
            .collect()
    }

    pub fn matrix(&self) -> [[f32; 4]; 4] {
        let s = self.scale;
        let tx = self.translation.0;
        let ty = self.translation.1;
        [
            [s,   0.0, 0.0, 0.0],
            [0.0, s,   0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [tx,  ty,  0.0, 1.0]
        ]
    }

    pub fn get_texture(&mut self, display: &glium::Display) -> &Texture2d {
        // We lazy load the textures, so the first time we ask for this we need to populate this
        if self.texture.is_none() {
            println!("Trying to open {}", &self.img);
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
