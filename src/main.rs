#[macro_use]
extern crate glium;
extern crate image;
extern crate spine;
extern crate cgmath;

use std::collections::HashMap;
use glium::{DisplayBuild, Surface};
use glium::index::PrimitiveType;
use spine::atlas::Atlas;
use cgmath::Matrix4;

use std::fs::File;
use std::io::BufReader;

mod run;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

/// insert all atlas textures into vertices
/// returns attachment (name, index) defining 4 consecutive vertices
fn load_atlas(atlas_src: &str, width: u32, height: u32,
              vertices: &mut Vec<Vertex>,
              indices: &mut Vec<u32>)
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
    for (name, t) in Atlas::from_file(atlas_src).expect("cannot find atlas file").into_iter() {

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

        indices.push(n);
        indices.push(n + 1);
        indices.push(n + 2);
        indices.push(n + 2);
        indices.push(n + 3);
        indices.push(n);

        n += 4;
    }
    textures
}

fn apply_sprite(sprites: &[spine::skeleton::animation::Sprite],
                attachments: &HashMap<String, u32>,
                vertices: &mut glium::VertexBuffer<Vertex>) {

    for (i, chunk) in vertices.map().chunks_mut(4).enumerate() {
        // search corresponding sprite
        if let Some(sprite) = attachments.iter().find(|&(_, n)| *n == 4*i as u32)
                     .and_then(|(name, _)| sprites.iter()
                        .find(|s| &s.attachment == name)) {

            let positions = sprite.positions;
            chunk[0].position[0] = positions[0][0];
            chunk[0].position[1] = positions[0][1];
            chunk[1].position[0] = positions[1][0];
            chunk[1].position[1] = positions[1][1];
            chunk[2].position[0] = positions[2][0];
            chunk[2].position[1] = positions[2][1];
            chunk[3].position[0] = positions[3][0];
            chunk[3].position[1] = positions[3][1];
            println!("modifying sprite '{}': {:?}", &sprite.attachment, chunk);
        }
    }

}

// fn draw_identity(texture_names: &HashMap<String, u32>, vertices: &mut Vec<Vertex>,
//     width: u32, height: u32) -> Vec<u32>
// {
//     let (width, height) = (width as f32, height as f32);
//     let tex_to_pos = |t: &[f32; 2]| {
//         [width * (2.0 * t[0] - 1.0), height * (2.0 * t[1] - 1.0)]
//     };
//
//     // Change all default position to texture position
//     for v in vertices.iter_mut() {
//         let pos = tex_to_pos(&v.tex_coords);
//         v.position = pos;
//     }
//
//     // build shapes by manipulating indices
//     // show all shapes for now
//     let mut indices = Vec::with_capacity(texture_names.len() * 6);
//     for (_, n) in texture_names.iter() {
//         indices.push(*n);
//         indices.push(*n + 1);
//         indices.push(*n + 2);
//         indices.push(*n + 2);
//         indices.push(*n + 3);
//         indices.push(*n);
//     }
//
//     indices
// }

fn main() {

    // TODO: parse program arguments
    let atlas_src = "/home/johann/projects/spine-render/example/spineboy.atlas";
    let atlas_img = "/home/johann/projects/spine-render/example/spineboy.png";
    let skeleton_src = "/home/johann/projects/spine-render/example/spineboy.json";

    // prepare glium objects
    let image = image::open(atlas_img).expect("couldn't read atlas image");
    let (width, height) = match image {
        image::DynamicImage::ImageRgba8(ref rgba) => (rgba.width(), rgba.height()),
        ref image => {
            let rgba = image.to_rgba();
            (rgba.width(), rgba.height())
        }
    };
    let window = glium::glutin::WindowBuilder::new().build_glium().unwrap();
    let texture = glium::texture::Texture2d::new(&window, image).unwrap();

    println!("texture: {:?}", texture, );

    // load atlas textures
    let (mut vertices, mut indices) = (Vec::new(), Vec::new());
    let texture_names = load_atlas(atlas_src, width, height, &mut vertices, &mut indices);

    // load skeleton from json
    let f = File::open(skeleton_src).ok().expect("Cannot open json file");
    let reader = BufReader::new(f);
    let skeleton = spine::skeleton::Skeleton::from_reader(reader).ok().expect("error while parsing json");
    let walk = skeleton.get_animated_skin("default", Some("walk")).ok().expect("error while creating animated skin");

    let sprites = walk.interpolate(0.0).expect("default skin should not be empty");

    // draw identity (for test)
    // let indices = draw_identity(&texture_names, &mut vertices, width, height);

    let mut vertex_buffer = glium::VertexBuffer::new(&window, &vertices).unwrap();
    let indices = glium::IndexBuffer::new(&window, PrimitiveType::TrianglesList, &indices).unwrap();

    // transform vertices
    apply_sprite(&sprites, &texture_names, &mut vertex_buffer);

    // opengl vertex program
    let vertex_src = r#"
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
    let fragment_src = r#"
        #version 140

        in vec2 v_tex_coords;
        out vec4 color;
        uniform sampler2D tex;

        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;

    let program = glium::Program::from_source(&window, vertex_src, fragment_src, None).unwrap();

    // the main loop, supposed to run with a constant FPS
    run::start_loop(|| {

        let mut target = window.draw();
        target.clear_color(0.0, 0.0, 1.0, 0.0);

        let perspective = {
            let (width, height) = target.get_dimensions();
            [
                [1.0 / width as f32, 0.0],
                [0.0, 1.0 / height as f32]
            ]
        };

        let uniforms = uniform! {
            tex: &texture,
            perspective: perspective
        };

        let params = glium::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            .. Default::default()
        };

        target.draw(&vertex_buffer, &indices, &program, &uniforms, &params).unwrap();
        target.finish().unwrap();

        for ev in window.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return run::Action::Stop,
                _ => ()
            }
        }

        run::Action::Continue
    });

        // for ev in window.wait_events() {
        //     match ev {
        //         glium::glutin::Event::Closed => return,
        //         _ => ()
        //     }
        // }
}
