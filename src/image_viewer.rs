use super::glium;
use glium::texture::{RawImage2d, Texture2d};
use glium::DrawError;

use glium::glutin::event::ElementState;
use glium::glutin::event::KeyboardInput;
use glium::glutin::event::VirtualKeyCode as Key;
use glium::glutin::event::WindowEvent;
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::event_loop::EventLoop;

use glium::uniforms::{EmptyUniforms, UniformsStorage};

use std::fs;
use std::iter::Iterator;
use std::str;
use std::time::Duration;
use std::time::Instant;

mod renderer;
use image_viewer::renderer::Renderable;
use image_viewer::renderer::Renderer;

use super::CODE_LOW;
use super::CODE_MID;
use super::CODE_HIGH;
use super::DL_DIR;
use super::SPEED_FAST;
use super::SPEED_MID;
use super::SPEED_SLOW;

extern crate image;

// Opens a new window, displaying only the files that currently exist in img
pub fn open_window() -> Result<(), DrawError> {
    let mut index = 0;
    let mut zoom = 1;
    let mut frame_time = SPEED_MID;

    // Do a bunch of init garbage
    let (display, events_loop) = create_display();
    let mut renderer = Renderer::new(display);
    let (bg_renderables, lc_renderables) = background_init(&renderer.display);
    let mut renderables = create_all_renderables_from_files(&renderer.display);
    let frame_time_nano = (frame_time * 1000000) as u64;
    let mut next_frame_time = Instant::now() + Duration::from_nanos(frame_time_nano);

    events_loop.run(move |ev, _, control_flow| {
        if let glium::glutin::event::Event::WindowEvent { event, .. } = ev {
            match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Released,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => match key {
                    Key::Down => frame_time = change_speed(frame_time, false),
                    Key::Up => frame_time = change_speed(frame_time, true),
                    Key::LBracket | Key::End => {
                        zoom = change_zoom(zoom, false);
                        if renderables[zoom].len() <= index {
                            index = 0;
                        };
                    }
                    Key::RBracket | Key::Home => {
                        zoom = change_zoom(zoom, true);
                        if renderables[zoom].len() <= index {
                            index = 0;
                        }
                    }
                    Key::Escape => {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    _ => (),
                },
                _ => {}
            }
        }

        if Instant::now() < next_frame_time {
            *control_flow = ControlFlow::WaitUntil(next_frame_time);
            return;
        }

        let frame_time_nano = (frame_time * 1000000) as u64;
        next_frame_time = Instant::now() + Duration::from_nanos(frame_time_nano);

        *control_flow = ControlFlow::WaitUntil(next_frame_time);

        renderer.new_frame();
        // Draw the background, then map overlay, then radar data
        renderer.draw(&bg_renderables[zoom]);
        renderer.draw(&lc_renderables[zoom]);
        renderer.draw(&renderables[zoom][index]);

        renderer.finish_frame();

        // Next image (with wraparound)
        index = {
            if index + 1 < renderables[zoom].len() {
                index + 1
            } else {
                0
            }
        };

        // Check for new images if we just wrapped around
        if index == 0 {
            add_all_new_renderables(&renderer.display, &mut renderables);
        }
    })
}

// Open a window and return the display and the associated events loop
fn create_display() -> (glium::Display, EventLoop<()>) {
    let events_loop = EventLoop::new();

    let window = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::PhysicalSize::new(512, 512))
        .with_title("Radar Monitor");

    let context = glium::glutin::ContextBuilder::new();

    let display =
        glium::Display::new(window, context, &events_loop).expect("Failed to create display");

    (display, events_loop)
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
fn background_init(display: &glium::Display) -> ([Renderable; 3], [Renderable; 3]) {
    // What is formatting
    (
        [
            renderable_from_image(display, &(CODE_LOW.to_string() + ".background.png")),
            renderable_from_image(display, &(CODE_MID.to_string() + ".background.png")),
            renderable_from_image(display, &(CODE_HIGH.to_string() + ".background.png")),
        ],
        [
            renderable_from_image(display, &(CODE_LOW.to_string() + ".locations.png")),
            renderable_from_image(display, &(CODE_MID.to_string() + ".locations.png")),
            renderable_from_image(display, &(CODE_HIGH.to_string() + ".locations.png")),
        ],
    )
}

fn create_all_renderables_from_files(display: &glium::Display) -> [Vec<Renderable>; 3] {
    let start = Instant::now();

    let renderables = [
        create_renderables_from_files(display, CODE_LOW),
        create_renderables_from_files(display, CODE_MID),
        create_renderables_from_files(display, CODE_HIGH),
    ];

    let end = Instant::now();
    let time = end.duration_since(start).as_millis();
    let num_renderables = (renderables[0].len() + renderables[1].len() + renderables[2].len()) as u128;
    let renderable_millis = time / num_renderables;

    println!(
        "Created {} renderables in {}ms ({} ms/renderable)",
        num_renderables, time, renderable_millis
    );

    renderables
}

fn create_renderables_from_files(display: &glium::Display, lc_code: &str) -> Vec<Renderable> {
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
            let r = renderable_from_image(display, &(dir.to_string() + e));
            let mut new_name = e.clone();
            new_name.remove(0);
            fs::rename(&(dir.to_string() + e), &(dir.to_string() + &new_name))
                .expect("Error renaming file");
            r
        })
        .collect()
}

fn add_all_new_renderables(display: &glium::Display, vecs: &mut [Vec<Renderable>; 3]) {
    add_new_renderables(display, &mut vecs[0], CODE_LOW);
    add_new_renderables(display, &mut vecs[1], CODE_MID);
    add_new_renderables(display, &mut vecs[2], CODE_HIGH);
}

fn add_new_renderables(display: &glium::Display, vec: &mut Vec<Renderable>, lc_code: &str) {
    let dir = &(DL_DIR.to_string() + lc_code + "/");
    let files = fs::read_dir(&dir).expect("Error reading image directory");

    let file_names: Vec<_> = files
        .map(|e| {
            e.expect("Error reading image filename")
                .file_name()
                .into_string()
                .expect("Error extracting image filename")
        })
        .collect();

    let mut file_names = file_names
        .iter()
        .filter(|e| e.starts_with('x'))
        .collect::<Vec<_>>();

    file_names.sort();

    for file_name in file_names {
        vec.push(renderable_from_image(display, &(dir.to_string() + file_name)));
        let mut new_name = file_name.clone();
        new_name.remove(0);
        fs::rename(
            &(dir.to_string() + file_name),
            &(dir.to_string() + &new_name),
        )
        .expect("Error renaming file");
    }
}

fn renderable_from_image(display: &glium::Display, img: &str) -> Renderable {
    let image = image::open(img)
        .expect("Error opening image file")
        .to_rgba8();

    let image_dim = image.dimensions();
    let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dim);
    let texture = Texture2d::new(display, image).expect("Error creating texture from image");

    Renderable { texture }
}
