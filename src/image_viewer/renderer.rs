use super::renderable::Renderable;
use glium::draw_parameters::{Blend, DrawParameters};
use glium::glutin::event_loop::EventLoop;
use glium::index::PrimitiveType;
use glium::texture::Texture2d;
use glium::uniforms::{EmptyUniforms, UniformsStorage};
use glium::{Display, Frame, IndexBuffer, Program, Surface, VertexBuffer};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    texture_pos: [f32; 2],
}

implement_vertex!(Vertex, position, texture_pos);

// Constants to define the vertices of the square
const VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-1.0, 1.0],
        texture_pos: [0.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0],
        texture_pos: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0],
        texture_pos: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0],
        texture_pos: [1.0, 0.0],
    },
];

const INDICES: [u16; 6] = [0, 2, 1, 1, 3, 2];

pub struct Renderer {
    pub display: Display, // Pub so outsiders can use it to create textures (maybe not a good idea)
    program: Program,
    vb: VertexBuffer<Vertex>,
    ib: IndexBuffer<u16>,

    target: Option<Frame>,
}

impl Renderer {
    pub fn new() -> (Self, EventLoop<()>) {
        let (display, events_loop) = create_display();
        let program = link_shader(&display);
        let (vb, ib) = create_buffers(&display);

        let renderer = Renderer {
            display,
            program,
            vb,
            ib,
            target: None,
        };

        (renderer, events_loop)
    }

    pub fn new_frame(&mut self) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        self.target = Some(target);
    }

    pub fn draw(&mut self, item: &mut Renderable) {
        let params = DrawParameters {
            blend: Blend::alpha_blending(),
            ..Default::default()
        };

        if let Some(target) = &mut self.target {
            let matrix = item.matrix();
            target
                .draw(
                    &self.vb,
                    &self.ib,
                    &self.program,
                    &uniforms(&item.get_texture(&self.display), matrix),
                    &params,
                )
                .expect("Error drawing BG");
        } else {
            panic!("Drew without a target, probably draw call without a new_frame call");
        }
    }

    pub fn finish_frame(&mut self) {
        if let Some(target) = self.target.take() {
            target.finish().expect("Frame Finishing Error");
            self.target = None;
        } else {
            panic!("Finished without a target, probably called without a new_frame call");
        }
    }
}

// Just a wrapper to be more readable at the draw call. This type signature is horrible...
fn uniforms(
    tex: &Texture2d,
    matrix: [[f32; 4]; 4],
) -> UniformsStorage<[[f32; 4]; 4], UniformsStorage<&Texture2d, EmptyUniforms>> {
    uniform! {
        tex: tex,
        matrix: matrix
    }
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

fn link_shader(display: &Display) -> Program {
    const VERT_SHADER: &str = include_str!("../res/shader.vert");
    const FRAG_SHADER: &str = include_str!("../res/shader.frag");
    Program::from_source(display, VERT_SHADER, FRAG_SHADER, None)
        .expect("Error creating shader program")
}

fn create_buffers(display: &Display) -> (VertexBuffer<Vertex>, IndexBuffer<u16>) {
    let vb = VertexBuffer::new(display, &VERTICES).expect("Error creating vertex buffer");
    let ib = IndexBuffer::new(display, PrimitiveType::TrianglesList, &INDICES)
        .expect("Error creating index buffer");

    (vb, ib)
}
