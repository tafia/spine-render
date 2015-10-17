#[macro_use]
extern crate glium;
extern crate image;

use std::collections::HashMap;
use glium::{DisplayBuild, Surface};

mod atlas;
mod run; // loop.rs

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

/// insert all atlas textures into vertices
/// returns attachment (name, index) defining 4 consecutive vertices
fn load_atlas(atlas_src: &str, width: u32, height: u32, vertices: &mut Vec<Vertex>)
    -> HashMap<String, u32>
{
    let (width, height) = (width as f32, height as f32);

    // helper closure to convert atlas texture coordinates into vertex texture's
    let to_tex_coords = |x: u16, y: u16| {
        let (x, y) = (x as f32, y as f32);
        [x / width, 1.0 - y / height]
    };

    // iterates over atlas textures and convert it to centered rectangle (4 vertices)
    let mut textures = HashMap::new();
    let mut n = vertices.len() as u32;
    for (name, t) in atlas::Atlas::from_file(atlas_src).into_iter() {
        let tex0 = to_tex_coords(t.xy.0,            t.xy.1);
        let tex1 = to_tex_coords(t.xy.0 + t.size.0, t.xy.1);
        let tex2 = to_tex_coords(t.xy.0 + t.size.0, t.xy.1 + t.size.1);
        let tex3 = to_tex_coords(t.xy.0,            t.xy.1 + t.size.1);
        // get 4 vertices defining rectangle texture, centered per default
        let (dx, dy) = (t.xy.0 as f32 / 2.0 / width, t.xy.1 as f32 / 2.0 / height);
        vertices.push(Vertex { position: [-dx, dy], tex_coords: tex0 });
        vertices.push(Vertex { position: [dx, dy], tex_coords: tex1 });
        vertices.push(Vertex { position: [dx, -dy], tex_coords: tex2 });
        vertices.push(Vertex { position: [-dx, -dy], tex_coords: tex3 });
        textures.insert(name, n);
        n += 4;
    }
    textures
}

fn draw_identity(texture_names: &HashMap<String, u32>, vertices: &mut Vec<Vertex>,
    width: u32, height: u32) -> Vec<u32>
{
    let (width, height) = (width as f32, height as f32);
    let tex_to_pos = |t: &[f32; 2]| {
        [width * (2.0 * t[0] - 1.0), height * (2.0 * t[1] - 1.0)]
    };

    // Change all default position to texture position
    for v in vertices.iter_mut() {
        let pos = tex_to_pos(&v.tex_coords);
        v.position = pos;
    }

    // build shapes by manipulating indices
    // show all shapes for now
    let mut indices = Vec::with_capacity(texture_names.len() * 6);
    for (_, n) in texture_names.iter() {
        indices.push(*n);
        indices.push(*n + 1);
        indices.push(*n + 2);
        indices.push(*n + 2);
        indices.push(*n + 3);
        indices.push(*n);
    }

    indices

}

fn main() {

    // TODO: parse program arguments
    let atlas_src = "/home/johann/projects/spine_render/example/spineboy.atlas";
    let atlas_img = "/home/johann/projects/spine_render/example/spineboy.png";

    // prepare glium objects
    let image = image::open(atlas_img).expect("couldn't read atlas image");
    let (width, height) = match image {
        image::DynamicImage::ImageRgba8(ref rgba) => (rgba.width(), rgba.height()),
        ref image => {
            let rgba = image.to_rgba();
            (rgba.width(), rgba.height())
        }
    };
    let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();
    let texture = glium::texture::Texture2d::new(&display, image).unwrap();

    // load textures
    let mut vertices: Vec<Vertex> = Vec::new();
    let texture_names = load_atlas(atlas_src, width, height, &mut vertices);

    // draw identity (for test)
    let indices = draw_identity(&texture_names, &mut vertices, width, height);

    let vertex_buffer = glium::VertexBuffer::new(&display, &vertices).unwrap();
    let indices = glium::IndexBuffer::new(&display,
                                          glium::index::PrimitiveType::TrianglesList,
                                          &indices).unwrap();


    // opengl vertex program
    let vertex_shader_src = r#"
        #version 140

        in vec2 position;
        in vec2 tex_coords;

        out vec2 v_tex_coords;

        uniform mat2 perspective;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = vec4(perspective * position, 0.0, 1.0);
        }
    "#;

    // opengl fragment program
    let fragment_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;
        out vec4 color;
        uniform sampler2D tex;

        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;

    let program = glium::Program::from_source(&display,
                                              vertex_shader_src,
                                              fragment_shader_src,
                                              None).unwrap();

    // the main loop, supposed to run with a constant FPS
    run::start_loop(|| {

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 0.0);

        let perspective = {
            let (width, height) = target.get_dimensions();
            // let aspect_ratio = height as f32 / width as f32 ;
            [
                [1.0 / width as f32, 0.0],
                [0.0, 1.0 / height as f32]
            ]
        };

        let uniforms = uniform! {
            tex: &texture,
            perspective: perspective
        };

        target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return run::Action::Stop,
                _ => ()
            }
        }

        run::Action::Continue
    });
}
