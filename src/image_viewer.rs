use super::glium;
use glium::DrawError;

use glium::glutin::event::ElementState;
use glium::glutin::event::KeyboardInput;
use glium::glutin::event::VirtualKeyCode as Key;
use glium::glutin::event::WindowEvent;
use glium::glutin::event_loop::ControlFlow;

use std::fs;
use std::iter::Iterator;
use std::str;
use std::time::Duration;
use std::time::Instant;

mod renderer;
mod renderable;
use image_viewer::renderable::Renderable;
use image_viewer::renderer::Renderer;

use super::CODE_LOW;
use super::CODE_MID;
use super::CODE_HIGH;
use super::DL_DIR;
use super::SPEED_FAST;
use super::SPEED_MID;
use super::SPEED_SLOW;

// Opens a new window, displaying only the files that currently exist in img
pub fn open_window() -> Result<(), DrawError> {
    let mut index = 0;
    let mut zoom = 1;
    let mut frame_time = SPEED_MID;

    // Do a bunch of init garbage
    let (mut renderer, events_loop) = Renderer::new();
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
            Renderable::from_disk_image(display, &(CODE_LOW.to_string() + ".background.png"), (-0.5, -0.5)),
            Renderable::from_disk_image(display, &(CODE_MID.to_string() + ".background.png"), (-0.5, -0.5)),
            Renderable::from_disk_image(display, &(CODE_HIGH.to_string() + ".background.png"), (-0.5, -0.5)),
        ],
        [
            Renderable::from_disk_image(display, &(CODE_LOW.to_string() + ".locations.png"), (0.5, -0.5)),
            Renderable::from_disk_image(display, &(CODE_MID.to_string() + ".locations.png"), (0.5, -0.5)),
            Renderable::from_disk_image(display, &(CODE_HIGH.to_string() + ".locations.png"), (0.5, -0.5)),
        ],
    )
}

fn create_all_renderables_from_files(display: &glium::Display) -> [Vec<Renderable>; 3] {
    let start = Instant::now();

    let renderables = [
        Renderable::from_location_folder(display, CODE_LOW),
        Renderable::from_location_folder(display, CODE_MID),
        Renderable::from_location_folder(display, CODE_HIGH),
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
        vec.push(Renderable::from_disk_image(display, &(dir.to_string() + file_name), (0.5, 0.5)));
        let mut new_name = file_name.clone();
        new_name.remove(0);
        fs::rename(
            &(dir.to_string() + file_name),
            &(dir.to_string() + &new_name),
        )
        .expect("Error renaming file");
    }
}