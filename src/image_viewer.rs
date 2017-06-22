use super::vulkano;
use super::vulkano_shader_derive;
extern crate winit;
extern crate vulkano_win;

use self::vulkano_win::VkSurfaceBuild;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::device::Device;
use vulkano::framebuffer::Framebuffer;
use vulkano::framebuffer::Subpass;
use vulkano::instance::Instance;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain;
use vulkano::swapchain::PresentMode;
use vulkano::swapchain::SurfaceTransform;
use vulkano::swapchain::Swapchain;
use vulkano::sync::now;
use vulkano::sync::GpuFuture;

use std::iter;

use std::str;
use std::fs;
use std::thread;
use std::time::{Instant, Duration};
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

extern crate image;

/*
#[derive(Copy, Clone)]
struct Vertex {
    pub position: [f32; 2],
    pub colour: [f32; 3],
    pub texture_pos: [f32; 2],
}

implement_vertex!(Vertex, position, colour, texture_pos);
*/

// Opens a new window, displaying only the files that currently exist in img
pub fn open_window(finish: &Arc<AtomicBool>, update: &Arc<AtomicBool>) {

    let mut index = 0;
    let mut force_redraw = false;
    let mut last_frame = Instant::now();
    let mut frame_time: usize = SPEED_MID;

    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("Failed to create Vulkan instance")
    };

    let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
        .next().expect("No device available");

    println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

    // Open the window
    let events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();

    let queue = physical.queue_families().find(|&q| {
        q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false)
    }).expect("Couldn't find a graphical queue family");

    //Initialise device
    let (device, mut queues) = {
        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        };

        Device::new(&physical, physical.supported_features(), &device_ext,
                    [(queue, 0.5)].iter().cloned()).expect("Failed to create device")
    };

    let queue = queues.next().unwrap();

    let (swapchain, images) = {
        let caps = window.surface().capabilities(physical)
            .expect("Failed to get surface capabilities");

        let dimensions = caps.current_extent.unwrap_or([1280, 1024]);

        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

        Swapchain::new(device.clone(), window.surface().clone(), caps.min_image_count, format,
                       dimensions, 1, caps.supported_usage_flags, &queue,
                       SurfaceTransform::Identity, alpha, PresentMode::Fifo, true, None)
                       .expect("Failed to create swapchain")
    };

    let vertex_buffer = {
        #[derive(Debug, Clone)]
        struct Vertex { position: [f32; 2] }
        impl_vertex!(Vertex, position);

        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), Some(queue.family()), [
                                       Vertex { position: [-0.5, 0.5] }, 
                                       Vertex { position: [-0.5, -0.5] }, 
                                       Vertex { position: [0.5, 0.5] }, 
                                       Vertex { position: [0.5, -0.5] } 
        ].iter().cloned()).expect("Failed to create buffer")
    };

    //Create shaders
    mod vs {
        #[derive(VulkanoShader)]
        #[ty = "vertex"]
        #[src = "
#version 450

layout(location = 0) in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}
"]
        struct Dummy;
    }

    mod fs {
        #[derive(VulkanoShader)]
        #[ty = "fragment"]
        #[src = "
#version 450

layout(location = 0) out vec4 f_colour;

void main() {
    f_colour = vec4(1.0, 0.0, 0.5, 1.0);
}
"]
        struct Dummy;
    }

    let vs = vs::Shader::load(&device).expect("Failed to create shader module");
    let fs = fs::Shader::load(&device).expect("Failed to create shader module");

    let render_pass = Arc::new(single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
        ).unwrap());

    let pipeline = Arc::new(GraphicsPipeline::start()
                            .vertex_input_single_buffer()
                            .vertex_shader(vs.main_entry_point(), ())
                            .triangle_list()
                            .viewports(iter::once(Viewport {
                                origin: [0.0, 0.0],
                                depth_range: 0.0 .. 1.0,
                                dimensions: [images[0].dimensions()[0] as f32, 
                                             images[0].dimensions()[1] as f32],
                            }))

                            .fragment_shader(fs.main_entry_point(), ())
                            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                            .build(device.clone())
                            .unwrap());

    let framebuffers = images.iter().map(|image| {
        Arc::new(Framebuffer::start(render_pass.clone())
                 .add(image.clone()).unwrap()
                 .build().unwrap())
    }).collect::<Vec<_>>();

    let mut previous_frame_end = Box::new(now(device.clone())) as Box<GpuFuture>;

    loop {

        previous_frame_end.cleanup_finished();

        let (image_num, acquire_future) = swapchain::acquire_next_image(swapchain.clone(),
                                                                       Duration::new(1, 0)).unwrap();

        // Build the render command
        let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap()
            .begin_render_pass(framebuffers[image_num].clone(), false,
                               vec![[0.0, 0.0, 1.0, 1.0].into()])
            .unwrap()
            .draw(pipeline.clone(), DynamicState::none(), vertex_buffer.clone(), (), ())
            .unwrap()
            .end_render_pass()
            .unwrap()
            .build().unwrap();

        let future = previous_frame_end.join(acquire_future)
            .then_execute(queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush().unwrap();

        previous_frame_end = Box::new(future) as Box<_>;

        // Handle events
        let mut done = false;
        events_loop.poll_events(|ev| {
            match ev {
                winit::Event::WindowEvent { event: winit::WindowEvent::Closed, .. } => done = true,
                _ => ()
            }
        });
        if done {return;}
    }
}
 /*

    let bg_textures = [texture_from_image(&display, 
                                          &(CODE_LOW.to_string() + ".background.png")),
                       texture_from_image(&display, 
                                          &(CODE_MID.to_string() + ".background.png")),
                       texture_from_image(&display, 
                                          &(CODE_HIGH.to_string() + ".background.png"))];
    let lc_textures = [texture_from_image(&display, 
                                          &(CODE_LOW.to_string() + ".locations.png")),
                       texture_from_image(&display, 
                                          &(CODE_MID.to_string() + ".locations.png")),
                       texture_from_image(&display, 
                                          &(CODE_HIGH.to_string() + ".locations.png"))];

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
                        Key::PageUp   => frame_time = change_speed(frame_time, true),
                        Key::PageDown => frame_time = change_speed(frame_time, false),
                        Key::LBracket | Key::End => {
                            zoom = change_zoom(zoom, false);
                            if textures[zoom].len() <= index {
                                index = 0;
                            };
                            force_redraw = true;
                        },
                        Key::RBracket | Key::Home => {
                            zoom = change_zoom(zoom, true);
                            if textures[zoom].len() <= index {
                                index = 0;
                            };
                            force_redraw = true;
                        },
                        _ => (),
                    }
                }
                _ => (),
            }
        }

        // Wait until the frame time has elapsed
        // 20 millisecond increments are ugly but reduce processor usage a lot and
        // don't seem to effect visual framerate
        let frame_time_nanos = (frame_time * 1000000) as u32;
        while !force_redraw && 
              (Instant::now() - last_frame).subsec_nanos() <= frame_time_nanos {
            thread::sleep(Duration::from_millis(20));
        }

        index = {
            if index + 1 < textures[zoom].len() {
                index + 1
            } else {
                0
            }
        };

        last_frame = Instant::now();
        force_redraw = false;

        // Check if we should update if we are looping over to the start again
        if index == 0 {
            let update = update.swap(false, Ordering::Relaxed);

            if update {
                add_all_new_textures(&display, &mut textures);
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
    } else if zoom == 2 {
        1
    } else {
        0
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
*/
