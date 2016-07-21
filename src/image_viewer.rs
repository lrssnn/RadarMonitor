use time;

use std::str;
use std::fs;
use std::iter::Iterator;
use std::sync::{Arc, Mutex};

use sfml::graphics::{Color, RenderTarget, RenderWindow, Texture, Sprite};
use sfml::window::{VideoMode, event, window_style, Key};

const DOWNLOAD_FOLDER: &'static str = "img/";
const LOCATION_CODE: &'static str = "IDR043";
// Opens a new window, displaying only the files that currently exist in img
pub fn open_window(finish: &Arc<Mutex<bool>>, update: &Arc<Mutex<bool>>){
    let mut current_index = 0;

    let mut last_frame = time::now();
    let mut this_frame;

    let time_per_frame = 200;


    let bg_texture = Texture::new_from_file(&(LOCATION_CODE.to_string() + ".background.png")).unwrap();
    let lc_texture = Texture::new_from_file(&(LOCATION_CODE.to_string() + ".locations.png")).unwrap();

    let bg = Sprite::new_with_texture(&bg_texture).unwrap();
    let lc = Sprite::new_with_texture(&lc_texture).unwrap();

    let mut window = RenderWindow::new(VideoMode::new_init(512, 512, 32),
                                        "Image Viewer",
					window_style::CLOSE,
					&Default::default())
	.unwrap();
    window.set_vertical_sync_enabled(true);

    loop {
        let textures = create_textures_from_files();
        let mut sprite = Sprite::new_with_texture(&textures[0]).unwrap();
	let mut reload = false;
        while !reload {
            for event in window.events() {
	        match event {
	            event::Closed => {exit(&finish); return},
		    event::KeyPressed { code: Key::Escape, .. } => {exit(&finish); return},
		    //event::KeyPressed { code: Key::PageUp, .. } => time_per_frame -= 100,
		    //event::KeyPressed { code: Key::PageDown, .. } => time_per_frame += 100,
                    _ => {}
                }
            }
	

	    window.clear(&Color::black());
	    window.draw(&bg);
	    window.draw(&lc);
	    window.draw(&sprite);
	    window.display();

	    this_frame = time::now();
	    if (this_frame - last_frame).num_milliseconds() >= time_per_frame {
	        current_index = next_image(&mut sprite, &textures, current_index);
	        last_frame = time::now();
	    
	        //Check if we should update if we are looping over to the start again
	        if current_index == 0 {
	            let update = update.lock().unwrap();
		    if *update {
		        reload = true;
		    }
	        }
	    }
        }
    }
}

fn exit(terminate: &Arc<Mutex<bool>>) {
    let mut terminate = terminate.lock().unwrap();  
    *terminate = true;
}

fn create_textures_from_files() -> Vec<Texture> {
    // Get a list of filenames in the folder
    let files = fs::read_dir("./img/").unwrap();
    let mut file_names: Vec<_> = files.map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    file_names.sort();

    let textures: Vec<Texture> = file_names.iter().map(|e| Texture::new_from_file(&(DOWNLOAD_FOLDER.to_string() + e)).unwrap()).collect();
    textures
}

fn next_image<'a>(sprite: &mut Sprite<'a>, textures: &'a Vec<Texture>, current_index: usize) -> usize{
    let index = if current_index + 1 < textures.len() { current_index + 1 } else { 0 };
    sprite.set_texture(&textures[index], true);
    index
}

