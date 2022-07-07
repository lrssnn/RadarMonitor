use glium::draw_parameters::{Blend, DrawParameters};
use glium::index::PrimitiveType;
use glium::texture::Texture2d;
use glium::{Display, Frame, IndexBuffer, Program, Surface, VertexBuffer};
use image_viewer::EmptyUniforms;
use image_viewer::UniformsStorage;

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

pub struct Renderable {
    pub texture: Texture2d,
}

pub struct Renderer {
    pub display: Display, // Pub so outsiders can use it to create textures (maybe not a good idea)
    program: Program,
    vb: VertexBuffer<Vertex>,
    ib: IndexBuffer<u16>,

    target: Option<Frame>,
}

impl Renderer {
    pub fn new(display: Display) -> Self {
        let program = link_shader(&display);
        let (vb, ib) = create_buffers(&display);

        Renderer { 
            display, 
            program, 
            vb, 
            ib, 
            target: None
        }
    }

    pub fn new_frame(&mut self) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        self.target = Some(target);
    }

    pub fn draw(&mut self, item: &Renderable) {
        let params = DrawParameters {
            blend: Blend::alpha_blending(),
            ..Default::default()
        };

        if let Some(target) = &mut self.target {
            target.draw(&self.vb, &self.ib, &self.program, &uniform_tex(&item.texture), &params).expect("Error drawing BG");
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

// Just a wrapper to be more readable at the draw call.
fn uniform_tex(tex: &Texture2d) -> UniformsStorage<&Texture2d, EmptyUniforms> {
    uniform! {
        tex: tex
    }
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