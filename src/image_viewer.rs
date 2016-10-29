use super::glium;
use glium::DisplayBuild;
use glium::{Surface, VertexBuffer, IndexBuffer};
use glium::index::PrimitiveType;
use glium::texture::{Texture2d, RawImage2d};

use glium::draw_parameters::{DrawParameters, Blend};
use time;

use std::str;
use std::fs;
use std::iter::Iterator;
use std::sync::{Arc};
use std::sync::atomic::{Ordering, AtomicBool};

use super::SPEED_SLOW;
use super::SPEED_MID;
use super::SPEED_FAST;
use super::LOCATION_CODE;
use super::DOWNLOAD_FOLDER;
use super::IMAGES_KEPT;

extern crate image;

#[derive(Copy, Clone)]
struct Vertex {
    pub position: [f32; 2],
    pub colour: [f32; 3],
    pub texture_pos: [f32; 2],
}

implement_vertex!(Vertex, position, colour, texture_pos);

fn texture_from_image(display: &glium::Display, img: &str) -> Texture2d {
    let image = image::open(img).unwrap().to_rgba();

    let image_dim = image.dimensions();

    let image = RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dim);

    Texture2d::new(display, image).unwrap()
    
}

// Opens a new window, displaying only the files that currently exist in img
pub fn open_window(finish: &Arc<AtomicBool>, update: &Arc<AtomicBool>){
    let mut current_index = 0;

    let mut last_frame = time::now();
    //let mut this_frame;


    let mut time_per_frame: usize = SPEED_MID;

    // Open the window
    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(512, 512)
        .with_title("Radar Monitor")
        .build_glium()
        .expect("Unable to create a window");

    let bg_texture = texture_from_image(&display, &(LOCATION_CODE.to_string() + 
                                                    ".background.png"));
    let lc_texture = texture_from_image(&display, &(LOCATION_CODE.to_string() + 
                                                    ".locations.png"));

    let program = {
        const VERT_SHADER: &'static str = include_str!("res/shader.vert");
        const FRAG_SHADER: &'static str = include_str!("res/shader.frag");

        glium::Program::from_source(&display, VERT_SHADER, FRAG_SHADER, None).unwrap()
    };

    let vertices = vec![
        Vertex { position: [-1.0,  1.0], colour: [0.0; 3], texture_pos: [0.0, 1.0]},
        Vertex { position: [-1.0, -1.0], colour: [0.0; 3], texture_pos: [0.0, 0.0]},
        Vertex { position: [ 1.0,  1.0], colour: [0.0; 3], texture_pos: [1.0, 1.0]},
        Vertex { position: [ 1.0, -1.0], colour: [0.0; 3], texture_pos: [1.0, 0.0]},
    ];

    let vertices = VertexBuffer::new(&display, &vertices).unwrap();

    let indices: Vec<u16> = vec![
        0, 2, 1,
        1, 3, 2,
    ];

    let indices = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &indices).unwrap();

    let params = DrawParameters {
        blend: Blend::alpha_blending(),
        ..Default::default()
    };


    loop {

        let mut target = display.draw();

        target.clear_color(0.0, 0.0, 0.0, 0.0);

        target.draw(&vertices,
                    &indices,
                    &program,
                    &uniform! {
                        tex: &bg_texture,
                    },
                    &Default::default())
            .unwrap();

        
        target.draw(&vertices,
                    &indices,
                    &program,
                    &uniform! {
                        tex: &lc_texture,
                    },
                    &params)
            .unwrap();

        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => {exit(finish); return},
                _ => (),
            }
        }
        /*
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
	            let update = update.load(Ordering::Relaxed);
		    if update {
		        reload = true;
		    }
	        }
	    }
        }
        */
    }
}


fn exit(terminate: &Arc<AtomicBool>) {
    terminate.store(true, Ordering::Relaxed);
}

/*
fn create_textures_from_files() -> Vec<Texture> {
    // Get a list of filenames in the folder
    let files = fs::read_dir(DOWNLOAD_FOLDER).unwrap();
    let mut file_names: Vec<_> = files.map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    file_names.sort();

    if IMAGES_KEPT > 0 {
        let len = file_names.len();
        let file_names = file_names.split_off(len - IMAGES_KEPT);
    }

    file_names.iter().map(|e| Texture::new_from_file(&(DOWNLOAD_FOLDER.to_string() + e)).unwrap()).collect()
}

fn next_image<'a>(sprite: &mut Sprite<'a>, textures: &'a Vec<Texture>, current_index: usize) -> usize{
    let index = if current_index + 1 < textures.len() { current_index + 1 } else { 0 };
    sprite.set_texture(&textures[index], true);
    index
}
*/

fn change_speed(current: usize, increase: bool) -> usize {
    if increase {
        if current != SPEED_SLOW { SPEED_FAST } else { SPEED_MID }
    } else if current != SPEED_FAST { SPEED_SLOW } else { SPEED_MID }    
}
