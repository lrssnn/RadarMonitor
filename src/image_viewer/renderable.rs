use glium::texture::RawImage2d;
use glium::texture::Texture2d;
use std::fs;

use super::DL_DIR;

pub struct Renderable {
    pub texture: Texture2d,
    pub translation: (f32, f32), // offset of the centre of the image, where the centre of the window is (0, 0)
    pub scale: f32, // scale where 1.0 is the entire window
}

impl Renderable {
    pub fn from_disk_image(display: &glium::Display, img: &str, translation: (f32, f32)) -> Self {
        let image = image::open(img)
            .expect("Error opening image file")
            .to_rgba8();

        let image_dim = image.dimensions();
        let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dim);
        let texture = Texture2d::new(display, image).expect("Error creating texture from image");

        Renderable { texture, translation, scale: 0.5 }
    }

    pub fn from_location_folder(display: &glium::Display, lc_code: &str) -> Vec<Self> {
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
                let r = Renderable::from_disk_image(display, &(dir.to_string() + e), (0.5, 0.5));
                let mut new_name = e.clone();
                new_name.remove(0);
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
}
