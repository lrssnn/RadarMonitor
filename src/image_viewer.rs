use super::glium;
use glium::{Surface, VertexBuffer, IndexBuffer};
use glium::index::PrimitiveType;
use glium::texture::{Texture2d, RawImage2d};

use glium::DrawError;
use glium::draw_parameters::{DrawParameters, Blend};

use glium::glutin::Event::WindowEvent;
use glium::glutin::KeyboardInput;
use glium::glutin::WindowEvent as Event;
use glium::glutin::VirtualKeyCode as Key;
use glium::glutin::ElementState;

use glium::uniforms::{UniformsStorage, EmptyUniforms};

use std::str;
use std::fs;
use std::thread;
use std::time::{Instant, Duration};
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
pub fn open_window(finish: &mpsc::Sender<()>, 
                   update: &mpsc::Receiver<()>)
                   -> Result<(), DrawError> {
    let mut index      = 0;
    let mut zoom       = 1;
    let mut last_frame = Instant::now();
    let mut frame_time = SPEED_MID;
    let mut sleep      = false;

    // Do a bunch of init garbage
    let (display, mut events_loop) = create_display();
    let (bg_textures, lc_textures) = background_init(&display);
    let program      = link_shader(&display);
    let (vb, ib)     = create_buffers(&display);
    let mut textures = create_all_textures_from_files(&display);

    let params = DrawParameters { blend: Blend::alpha_blending(), ..Default::default() };

    loop {
        if !sleep {
            // Grab the display and clear it
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 0.0);
            
            // Draw the background, then map overlay, then radar data
            target.draw(&vb, &ib, &program, &uniform_tex(&bg_textures[zoom]), &params)?;
            target.draw(&vb, &ib, &program, &uniform_tex(&lc_textures[zoom]), &params)?;
            target.draw(&vb, &ib, &program, &uniform_tex(&textures[zoom][index]), &params)?;

            target.finish().expect("Frame Finishing Error");


            // Wait until the frame time has elapsed
            // 20 millisecond increments are ugly but reduce processor usage a lot and
            // don't seem to effect visual framerate
            // This is so ugly
            // Clearly I don't understand closures because I was not expecting these side effects
            // (speed and zoom) to actually make it out of the closure????
            let frame_time_nanos = (frame_time * 1000000) as u32;
            let mut force_redraw = false;
            while !force_redraw && 
                (Instant::now() - last_frame).subsec_nanos() <= frame_time_nanos {
                let mut done = false;
                events_loop.poll_events(|ev| {
                    // Unwrap into a WindowEvent because we don't care about any DeviceEnvents
                    if let WindowEvent { event: e, .. } = ev {
                        match e {
                            Event::Closed => done = true,

                            Event::KeyboardInput {
                                input: KeyboardInput {
                                    state: ElementState::Released,
                                    virtual_keycode: Some(key),
                                    ..
                                },
                                ..
                            } => {
                                match key {
                                    Key::Escape   => done = true,
                                    Key::PageDown => frame_time = change_speed(frame_time, false),
                                    Key::PageUp   => frame_time = change_speed(frame_time, true),
                                    Key::LBracket | Key::End => {
                                        zoom = change_zoom(zoom, false);
                                        force_redraw = true;
                                        if textures[zoom].len() <= index {
                                            index = 0;
                                        };
                                    },
                                    Key::RBracket | Key::Home => {
                                        zoom = change_zoom(zoom, true);
                                        force_redraw = true;
                                        if textures[zoom].len() <= index {
                                            index = 0;
                                        }
                                    },
                                    Key::Back => sleep = true,
                                    _ => (),
                                }
                            }
                            _ => (),
                        }
                    }
                });

                if done {
                    exit(finish);
                    return Ok(());
                }
                thread::sleep(Duration::from_millis(20));
            }

            // Next image (with wraparound)
            index = {
                if index + 1 < textures[zoom].len() {
                    index + 1
                } else {
                    0
                }
            };

            last_frame = Instant::now();

            // Check for new images if we just wrapped around
            if index == 0 && update.try_recv().is_ok() {
                add_all_new_textures(&display, &mut textures);
            }
        
        } else {
            // We are asleep, just check for close or unsleep, and delay for a long time
            let mut done = false;
            events_loop.poll_events(|ev| {
                use self::Key::*;
                if let WindowEvent { event: e, .. } = ev {
                    match e {
                        Event::Closed => done = true,

                        Event::KeyboardInput {
                            input: KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(key),
                                ..
                            },
                            ..
                        } => {
                            match key {
                                Escape => done  = true,
                                Enter  => sleep = false,
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                }
            });

            if done {
                exit(finish);
                return Ok(());
            }
            thread::sleep(Duration::from_millis(1000));
        }
    }

}

// Just a wrapper to be more readable at the draw call. 
fn uniform_tex(tex: &Texture2d) -> UniformsStorage<&Texture2d, EmptyUniforms> {
    uniform! {
        tex: tex
    }
}

// Open a window and return the display and the associated events loop
fn create_display() -> (glium::Display, glium::glutin::EventsLoop) {
    let events_loop = glium::glutin::EventsLoop::new();

    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(512, 512)
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
    let image = image::open(img).expect("Error opening image file").to_rgba();

    let image_dim = image.dimensions();

    let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dim);

    Texture2d::new(display, image).expect("Error creating texture from image")

}

