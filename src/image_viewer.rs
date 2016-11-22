use super::glium;
use glium::DisplayBuild;
use glium::{Surface, VertexBuffer, IndexBuffer};
use glium::index::PrimitiveType;
use glium::texture::{Texture2d, RawImage2d};

use glium::draw_parameters::{DrawParameters, Blend};

use glium::glutin::Event::KeyboardInput;
use glium::glutin::VirtualKeyCode as Key;
use glium::glutin::ElementState;

use time;

use std::str;
use std::fs;
use std::iter::Iterator;
use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicBool};

use super::SPEED_SLOW;
use super::SPEED_MID;
use super::SPEED_FAST;
use super::CODE_LOW;
use super::CODE_MID;
use super::CODE_HIGH;
use super::DL_DIR;
use super::IMAGES_KEPT;

extern crate image;

#[derive(Copy, Clone)]
struct Vertex {
    pub position: [f32; 2],
    pub colour: [f32; 3],
    pub texture_pos: [f32; 2],
}

implement_vertex!(Vertex, position, colour, texture_pos);

// Opens a new window, displaying only the files that currently exist in img
pub fn open_window(finish: &Arc<AtomicBool>, update: &Arc<AtomicBool>) {

    let mut index = 0;
    let mut last_frame = time::now();
    let mut frame_time: usize = SPEED_MID;

    // Open the window
    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(512, 512)
        .with_title("Radar Monitor")
        .build_glium()
        .expect("Unable to create a window");

    let bg_textures = [
        texture_from_image(&display, &(CODE_LOW.to_string() + ".background.png")),
        texture_from_image(&display, &(CODE_MID.to_string() + ".background.png")),
        texture_from_image(&display, &(CODE_HIGH.to_string() + ".background.png")),
    ];
    let lc_textures = [
        texture_from_image(&display, &(CODE_LOW.to_string() + ".locations.png")),
        texture_from_image(&display, &(CODE_MID.to_string() + ".locations.png")),
        texture_from_image(&display, &(CODE_HIGH.to_string() + ".locations.png")),
    ];

    let program = {
        const VERT_SHADER: &'static str = include_str!("res/shader.vert");
        const FRAG_SHADER: &'static str = include_str!("res/shader.frag");

        glium::Program::from_source(&display, VERT_SHADER, FRAG_SHADER, None)
            .expect("Error creating shader program")
    };

    let vertices = vec![
        Vertex { position: [-1.0,  1.0], colour: [0.0; 3], texture_pos: [0.0, 1.0]},
        Vertex { position: [-1.0, -1.0], colour: [0.0; 3], texture_pos: [0.0, 0.0]},
        Vertex { position: [ 1.0,  1.0], colour: [0.0; 3], texture_pos: [1.0, 1.0]},
        Vertex { position: [ 1.0, -1.0], colour: [0.0; 3], texture_pos: [1.0, 0.0]},
    ];

    let vertices = VertexBuffer::new(&display, &vertices).expect("Error creating vertex buffer");

    let indices: Vec<u16> = vec![
        0, 2, 1,
        1, 3, 2,
    ];

    let indices = IndexBuffer::new(&display, PrimitiveType::TrianglesList, &indices)
        .expect("Error creating index buffer");

    let params = DrawParameters { blend: Blend::alpha_blending(), ..Default::default() };

    let mut textures = create_all_textures_from_files(&display);

    let mut zoom = 1;

    loop {
        let mut target = display.draw();

        target.clear_color(0.0, 0.0, 0.0, 0.0);

        target.draw(&vertices,
                  &indices,
                  &program,
                  &uniform! {
                        tex: &bg_textures[zoom],
                    },
                  &Default::default())
            .expect("Drawing Error");


        target.draw(&vertices,
                  &indices,
                  &program,
                  &uniform! {
                        tex: &lc_textures[zoom],
                    },
                  &params)
            .expect("Drawing Error");

        target.draw(&vertices,
                  &indices,
                  &program,
                  &uniform! {
                        tex: &textures[zoom][index],
                    },
                  &params)
            .expect("Drawing Error");

        target.finish().expect("Frame Finishing Error");

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => {
                    exit(finish);
                    return;
                }

                KeyboardInput(ElementState::Released, _, Some(key)) => {
                    match key {
                        Key::Escape => {
                            exit(finish);
                            return;
                        }
                        Key::PageUp => frame_time = change_speed(frame_time, true),
                        Key::PageDown => frame_time = change_speed(frame_time, false),
                        Key::LBracket => zoom = change_zoom(zoom, false),
                        Key::RBracket => zoom = change_zoom(zoom, true),
                        Key::End => zoom = change_zoom(zoom, false),
                        Key::Home => zoom = change_zoom(zoom, true),
		    	_ => ()
                    }
                }
                ev => (println!("{:?}", ev)),
            }
        }

        if (time::now() - last_frame).num_milliseconds() >= frame_time as i64 {
            index = {
                if index + 1 < textures[zoom].len() {
                    index + 1
                } else {
                    0
                }
            };

            last_frame = time::now();

            // Check if we should update if we are looping over to the start again
            if index == 0 {
                let update = update.swap(false, Ordering::Relaxed);

                if update {
                    add_all_new_textures(&display, &mut textures);
                }
            }
        }
    }
}


fn exit(terminate: &Arc<AtomicBool>) {
    terminate.store(true, Ordering::Relaxed);
}

fn change_zoom(zoom: usize, faster: bool) -> usize {
   if faster {
       if zoom == 0 {
           1 
       } else {
           2
       }
   } else {
       if zoom == 2 {
           1
       } else {
           0 
       }
   }
}

fn create_all_textures_from_files(display: &glium::Display) -> [Vec<Texture2d>; 3] {
    [create_textures_from_files(display, CODE_LOW),
    create_textures_from_files(display, CODE_MID),
    create_textures_from_files(display, CODE_HIGH)]
}

fn create_textures_from_files(display: &glium::Display, lc_code: &str) -> Vec<Texture2d> {
    let dir = &(DL_DIR.to_string() + lc_code + "/");
    let files = fs::read_dir(dir).expect("Error reading image directory");
    let mut file_names: Vec<_> = files.map(|e| {
            e.expect("Error reading image filename")
                .file_name()
                .into_string()
                .expect("Error extracting image filename")
        })
        .collect();

    file_names.sort();

    if IMAGES_KEPT > 0 {
        let len = file_names.len();
        file_names = file_names.split_off(len - IMAGES_KEPT);
    }

    file_names.iter()
        .map(|e| {
            let r = texture_from_image(display, &(dir.to_string() + e));
            let mut new_name = e.clone();
            new_name.remove(0);
            fs::rename(&(dir.to_string() + e), 
                       &(dir.to_string() + &new_name))
                .expect("Error renaming file");
            r
        })
        .collect()
}

fn add_all_new_textures(display: &glium::Display, vecs: &mut [Vec<Texture2d>; 3]) {
    add_new_textures(display, &mut vecs[0], CODE_LOW);
    add_new_textures(display, &mut vecs[1], CODE_MID);
    add_new_textures(display, &mut vecs[2], CODE_HIGH);
}

fn add_new_textures(display: &glium::Display, vec: &mut Vec<Texture2d>, lc_code: &str) {
    let dir = &(DL_DIR.to_string() + lc_code + "/");
    let files = fs::read_dir(&dir)
        .expect("Error reading image directory");

    let file_names: Vec<_> = files.map(|e| {
            e.expect("Error reading image filename")
                .file_name()
                .into_string()
                .expect("Error extracting image filename")
        })
        .collect();

    let mut file_names = file_names.iter().filter(|e| e.starts_with('x')).collect::<Vec<_>>();

    file_names.sort();

    for file_name in file_names {
        vec.push(texture_from_image(display, &(dir.to_string() + file_name)));
        let mut new_name = file_name.clone();
        new_name.remove(0);
        fs::rename(&(dir.to_string() + file_name),
                   &(dir.to_string() + &new_name))
            .expect("Error renaming file");
    }
}

fn change_speed(current: usize, increase: bool) -> usize {
    if increase {
        if current == SPEED_SLOW {
            SPEED_MID
        } else {
            SPEED_FAST
        }
    } else if current == SPEED_FAST {
        SPEED_MID
    } else {
        SPEED_SLOW
    }
}

fn texture_from_image(display: &glium::Display, img: &str) -> Texture2d {
    let image = image::open(img).expect("Error opening image file").to_rgba();

    let image_dim = image.dimensions();

    let image = RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dim);

    Texture2d::new(display, image).expect("Error creating texture from image")

}
