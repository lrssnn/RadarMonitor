use super::glium;
use glium::{Surface, VertexBuffer, IndexBuffer};
use glium::index::PrimitiveType;
use glium::texture::{Texture2d, RawImage2d};

use glium::DrawError;
use glium::draw_parameters::{DrawParameters, Blend};

use glium::glutin::event_loop::EventLoop;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::event::WindowEvent;
use glium::glutin::event::KeyboardInput;
use glium::glutin::event::VirtualKeyCode as Key;
use glium::glutin::event::ElementState;

use glium::uniforms::{UniformsStorage, EmptyUniforms};

use std::str;
use std::fs;
use std::iter::Iterator;
use std::sync::mpsc;

use super::SPEED_SLOW;
use super::SPEED_MID;
use super::SPEED_FAST;
use super::CODE_LOW;
use super::CODE_MID;
use super::CODE_HIGH;
use super::DL_DIR;

extern crate image;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    texture_pos: [f32; 2],
}

implement_vertex!(Vertex, position, texture_pos);

// Constants to define the vertices of the square
const VERTICES: [Vertex; 4] = [
    Vertex { position: [-1.0,  1.0], texture_pos: [0.0, 1.0]},
    Vertex { position: [-1.0, -1.0], texture_pos: [0.0, 0.0]},
    Vertex { position: [ 1.0,  1.0], texture_pos: [1.0, 1.0]},
    Vertex { position: [ 1.0, -1.0], texture_pos: [1.0, 0.0]},
];

const INDICES: [u16; 6] = [0, 2, 1, 1, 3, 2];

// Opens a new window, displaying only the files that currently exist in img
pub fn open_window() -> Result<(), DrawError> {
    let mut index      = 0;
    let mut zoom       = 1;
    let mut frame_time = SPEED_MID;

    // Do a bunch of init garbage
    let (display, events_loop) = create_display();
    let (bg_textures, lc_textures) = background_init(&display);
    let program      = link_shader(&display);
    let (vb, ib)     = create_buffers(&display);
    let mut textures = create_all_textures_from_files(&display);

    let params = DrawParameters { blend: Blend::alpha_blending(), ..Default::default() };

    events_loop.run(move |ev, _, control_flow| {
        // Grab the display and clear it
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        
        // Draw the background, then map overlay, then radar data
        target.draw(&vb, &ib, &program, &uniform_tex(&bg_textures[zoom]), &params).expect("Error drawing BG");
        target.draw(&vb, &ib, &program, &uniform_tex(&lc_textures[zoom]), &params).expect("Error drawing Overlay");
        target.draw(&vb, &ib, &program, &uniform_tex(&textures[zoom][index]), &params).expect("Error drawing data");

        target.finish().expect("Frame Finishing Error");

        let frame_time_nano = (frame_time * 1000000) as u64;
        println!("{} millis per frame = {} nanos per frame...", frame_time, frame_time_nano);

        let next_frame_time = std::time::Instant::now() + std::time::Duration::from_nanos(frame_time_nano);
        println!("next frame at {:?}", next_frame_time);
        *control_flow = ControlFlow::WaitUntil(next_frame_time);
        
        // Next image (with wraparound)
        index = {
            if index + 1 < textures[zoom].len() {
                index + 1
            } else {
                0
            }
        };

        // Check for new images if we just wrapped around
        if index == 0 {
            add_all_new_textures(&display, &mut textures);
        }

        match ev {
            glium::glutin::event::Event::WindowEvent { event, .. } => match event {

                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },

                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(key),
                        ..
                    },
                    ..
                } => {
                    match key {
                        Key::Down => frame_time = change_speed(frame_time, false),
                        Key::Up   => frame_time = change_speed(frame_time, true),
                        Key::LBracket | Key::End => {
                            zoom = change_zoom(zoom, false);
                            if textures[zoom].len() <= index {
                                index = 0;
                            };
                        },
                        Key::RBracket | Key::Home => {
                            zoom = change_zoom(zoom, true);
                            if textures[zoom].len() <= index {
                                index = 0;
                            }
                        },
                        Key::Escape => *control_flow = ControlFlow::Exit,
                        _ => (),
                    }
                }
                _ => (),
            },
            _ => (),
        }
    })
}

// Just a wrapper to be more readable at the draw call. 
fn uniform_tex(tex: &Texture2d) -> UniformsStorage<&Texture2d, EmptyUniforms> {
    uniform! {
        tex: tex
    }
}

// Open a window and return the display and the associated events loop
fn create_display() -> (glium::Display, EventLoop<()>) {
    let events_loop = EventLoop::new();

    let window = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::PhysicalSize::new(512, 512))
        .with_title("Radar Monitor");

    let context = glium::glutin::ContextBuilder::new();

    let display = glium::Display::new(window, context, &events_loop)
        .expect("Failed to create display");

    (display, events_loop)
}

fn link_shader(display: &glium::Display) -> glium::Program {
    const VERT_SHADER: &'static str = include_str!("res/shader.vert");
    const FRAG_SHADER: &'static str = include_str!("res/shader.frag");
    glium::Program::from_source(display, VERT_SHADER, FRAG_SHADER, None)
        .expect("Error creating shader program")
}

fn create_buffers(display: &glium::Display) -> (VertexBuffer<Vertex>, IndexBuffer<u16>) {
    let vb = VertexBuffer::new(display, &VERTICES)
        .expect("Error creating vertex buffer");
    let ib = IndexBuffer::new(display, PrimitiveType::TrianglesList, &INDICES)
        .expect("Error creating index buffer");

    (vb, ib)
}

fn exit(terminate: &mpsc::Sender<()>) {
    terminate.send(()).unwrap();
}

fn change_zoom(zoom: usize, faster: bool) -> usize {
    if faster {
        if zoom == 0 {
            1
        } else {
            2
        }
    } else if zoom == 2 {
        1
    } else {
        0
    }
}

fn change_speed(current: usize, increase: bool) -> usize {
    let value = if increase {
        if current == SPEED_SLOW {
            SPEED_MID
        } else {
            SPEED_FAST
        }
    } else if current == SPEED_FAST {
        SPEED_MID
    } else {
        SPEED_SLOW
    };
    println!("Frame Time = {}", value);
    value
}

// Create background and location texture arrays. Just to clean up init in main function
fn background_init(display: &glium::Display) -> ([Texture2d; 3], [Texture2d; 3]) {
    // What is formatting
    ([texture_from_image(display, &(CODE_LOW.to_string()  + ".background.png")),
      texture_from_image(display, &(CODE_MID.to_string()  + ".background.png")),
      texture_from_image(display, &(CODE_HIGH.to_string() + ".background.png"))],
     [texture_from_image(display, &(CODE_LOW.to_string()  + ".locations.png")),
      texture_from_image(display, &(CODE_MID.to_string()  + ".locations.png")),
      texture_from_image(display, &(CODE_HIGH.to_string() + ".locations.png"))
    ])
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

    file_names.iter()
        .map(|e| {
            let r = texture_from_image(display, &(dir.to_string() + e));
            let mut new_name = e.clone();
            new_name.remove(0);
            fs::rename(&(dir.to_string() + e), &(dir.to_string() + &new_name))
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
    let files = fs::read_dir(&dir).expect("Error reading image directory");

    let file_names: Vec<_> = files.map(|e| {
            e.expect("Error reading image filename")
                .file_name()
                .into_string()
                .expect("Error extracting image filename")
        })
        .collect();

    let mut file_names = file_names.iter().filter(|e| 
        e.starts_with('x')
    ).collect::<Vec<_>>();

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


fn texture_from_image(display: &glium::Display, img: &str) -> Texture2d {
    let image = image::open(img).expect("Error opening image file").to_rgba8();

    let image_dim = image.dimensions();

    let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dim);

    Texture2d::new(display, image).expect("Error creating texture from image")
}

