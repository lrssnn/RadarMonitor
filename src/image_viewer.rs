use time;

use std::str;
use std::fs;
use std::iter::Iterator;
use std::sync::{Arc, Mutex};

use sfml::graphics::{Color, RenderTarget, RenderWindow, Texture, Sprite};
use sfml::window::{VideoMode, event, window_style, Key};


use super::SPEED_SLOW;
use super::SPEED_MID;
use super::SPEED_FAST;
use super::LOCATION_CODE;
use super::DOWNLOAD_FOLDER;
use super::IMAGES_KEPT;

// Opens a new window, displaying only the files that currently exist in img
pub fn open_window(finish: &Arc<Mutex<bool>>, update: &Arc<Mutex<bool>>){
    let mut current_index = 0;

    let mut last_frame = time::now();
    let mut this_frame;


    let mut time_per_frame: usize = SPEED_MID;


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
		    event::KeyPressed { code: Key::PageUp, .. } => time_per_frame = change_speed(time_per_frame, true),
		    event::KeyPressed { code: Key::PageDown, .. } => time_per_frame = change_speed(time_per_frame, false),
                    _ => {}
                }
            }
	

	    window.clear(&Color::black());
	    window.draw(&bg);
	    window.draw(&lc);
	    window.draw(&sprite);
	    window.display();

	    this_frame = time::now();
	    if (this_frame - last_frame).num_milliseconds() >= time_per_frame as i64 {
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
    let files = fs::read_dir(DOWNLOAD_FOLDER).unwrap();
    let mut file_names: Vec<_> = files.map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    file_names.sort();

    if IMAGES_KEPT > 0 {
        let len = file_names.len();
        let mut file_names = file_names.split_off(len - IMAGES_KEPT);
    }

    let textures: Vec<Texture> = file_names.iter().map(|e| Texture::new_from_file(&(DOWNLOAD_FOLDER.to_string() + e)).unwrap()).collect();
    textures
}

fn next_image<'a>(sprite: &mut Sprite<'a>, textures: &'a Vec<Texture>, current_index: usize) -> usize{
    let index = if current_index + 1 < textures.len() { current_index + 1 } else { 0 };
    sprite.set_texture(&textures[index], true);
    index
}

fn change_speed(current: usize, increase: bool) -> usize {
    if increase {
        if current == SPEED_FAST {
           return SPEED_MID;
        } else {
	    return SPEED_SLOW;
	}
    } else {
        if current == SPEED_SLOW {
	    return SPEED_MID;
	} else {
	    return SPEED_FAST;
	}
    } return SPEED_MID;
}
