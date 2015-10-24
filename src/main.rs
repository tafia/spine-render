#[macro_use]
extern crate glium;
extern crate image;
extern crate spine;

use std::collections::HashMap;
use glium::{DisplayBuild, Surface};
use glium::index::PrimitiveType;
use spine::atlas::Atlas;

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
    for (name, t) in Atlas::from_file(atlas_src).expect("cannot find atlas file").into_iter() {

        let tex0 = to_tex_coords(t.xy.0,            t.xy.1);
        let tex1 = to_tex_coords(t.xy.0 + t.size.0, t.xy.1);
        let tex2 = to_tex_coords(t.xy.0 + t.size.0, t.xy.1 + t.size.1);
        let tex3 = to_tex_coords(t.xy.0,            t.xy.1 + t.size.1);
        // get 4 vertices defining rectangle texture, centered per default
        if t.rotate {
            vertices.push(Vertex { position: [0.0, 0.0], tex_coords: tex3 });
            vertices.push(Vertex { position: [0.0, 0.0], tex_coords: tex0 });
            vertices.push(Vertex { position: [0.0, 0.0], tex_coords: tex1 });
            vertices.push(Vertex { position: [0.0, 0.0], tex_coords: tex2 });
        } else {
            vertices.push(Vertex { position: [0.0, 0.0], tex_coords: tex0 });
            vertices.push(Vertex { position: [0.0, 0.0], tex_coords: tex1 });
            vertices.push(Vertex { position: [0.0, 0.0], tex_coords: tex2 });
            vertices.push(Vertex { position: [0.0, 0.0], tex_coords: tex3 });
        }
        textures.insert(name, n);

        n += 4;
    }
    textures
}

fn apply_sprite(sprites: &[spine::skeleton::animation::Sprite],
                attachments: &HashMap<String, u32>,
                vertices: &mut glium::VertexBuffer<Vertex>) -> Vec<u32> {

    let mut indices = Vec::new();
    let mut vertices = vertices.map();
    for sprite in sprites {
        if let Some(&n) = attachments.get(&sprite.attachment) {

            indices.push(n);
            indices.push(n + 1);
            indices.push(n + 2);
            indices.push(n + 2);
            indices.push(n + 3);
            indices.push(n);

            let n = n as usize;
            let positions = sprite.positions;
            vertices[n].position = positions[0];
            vertices[n + 1].position = positions[1];
            vertices[n + 2].position = positions[2];
            vertices[n + 3].position = positions[3];
        }
    }
    indices
}

fn main() {

    // TODO: parse program arguments
    let atlas_src = "/home/johann/projects/spine_render/example/spineboy.atlas";
    let atlas_img = "/home/johann/projects/spine_render/example/spineboy.png";
    let skeleton_src = "/home/johann/projects/spine_render/example/spineboy.json";

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

    // load atlas textures
    let mut vertices = Vec::new();
    let texture_names = load_atlas(atlas_src, width, height, &mut vertices);
    let mut vertex_buffer = glium::VertexBuffer::new(&window, &vertices).unwrap();

    // load skeleton from json
    let f = File::open(skeleton_src).ok().expect("Cannot open json file");
    let reader = BufReader::new(f);
    let skeleton = spine::skeleton::Skeleton::from_reader(reader).ok().expect("error while parsing json");

    // let walk = skeleton.get_animated_skin("default", Some("walk")).ok().expect("error while creating animated skin");
    // let walk = skeleton.get_animated_skin("default", Some("hit")).ok().expect("error while creating animated skin");
    // let walk = skeleton.get_animated_skin("default", Some("run")).ok().expect("error while creating animated skin");
    // let walk = skeleton.get_animated_skin("default", Some("death")).ok().expect("error while creating animated skin");
    let walk = skeleton.get_animated_skin("default", Some("jump")).ok().expect("error while creating animated skin");
    // let walk = skeleton.get_animated_skin("default", Some("idle")).ok().expect("error while creating animated skin");
    // let walk = skeleton.get_animated_skin("default", None).ok().expect("error while creating animated skin");

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

    let params = glium::DrawParameters {
        blend: glium::Blend::alpha_blending(),
        .. Default::default()
    };

    let mut perspective = [[0.0, 0.0], [0.0, 0.0]];

    let program = glium::Program::from_source(&window, vertex_src, fragment_src, None).unwrap();
    let delta = 0.01;
    let mut iter = walk.iter(delta);

    // let sprites = iter.next().unwrap();

    // the main loop, supposed to run with a constant FPS
    run::start_loop(|| {
        // for _ in 0..5 {
        if let Some(sprites) = iter.next() {

            // println!("sprite: {:#?}", sprites);

            let indices = apply_sprite(&sprites, &texture_names, &mut vertex_buffer);
            let indices = glium::IndexBuffer::new(&window, PrimitiveType::TrianglesList, &indices).unwrap();

            let mut target = window.draw();
            target.clear_color(0.0, 0.0, 1.0, 0.0);

            let (width, height) = target.get_dimensions();
            perspective[0][0] = 1.0 / width as f32;
            perspective[1][1] = 1.0 / height as f32;

            let uniforms = uniform! {
                tex: &texture,
                perspective: perspective
            };

            target.draw(&vertex_buffer, &indices, &program, &uniforms, &params).unwrap();
            target.finish().unwrap();

            for ev in window.poll_events() {
                match ev {
                    glium::glutin::Event::Closed => return run::Action::Stop,
                    _ => ()
                }
            }
        } else {
            iter = walk.iter(delta);
        }
// }

        run::Action::Continue
        // run::Action::Stop
    });
}
