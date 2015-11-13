#[macro_use]
extern crate glium;
extern crate image;
extern crate spine;

use std::collections::HashMap;
use glium::{DisplayBuild, Surface};
use glium::index::PrimitiveType;
use spine::atlas::{Atlas, AtlasError};
use spine::skeleton::Skeleton;

use std::fs::File;
use std::io::BufReader;

mod run;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

struct AtlasItems {
    textures: HashMap<String, usize>,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

/// insert all atlas textures into vertices
/// returns attachment (name, index) defining 4 consecutive vertices
fn read_atlas(atlas_src: &str, width: u32, height: u32, skeleton: &Skeleton, skin_name: &str)
    -> Result<AtlasItems, AtlasError>
{

    let attachments = skeleton.get_skin(skin_name).unwrap().attachment_positions();
    let (width, height) = (width as f32, height as f32);

    // helper closure to convert atlas texture coordinates into vertex texture's
    let to_tex_coords = |x: u16, y: u16| {
        let (x, y) = (x as f32, y as f32);
        [x / width, 1.0 - y / height]
    };

    // iterates over atlas textures and convert it to centered rectangle (4 vertices)
    let mut textures = HashMap::new();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let atlas_file = try!(File::open(atlas_src)
        .map_err(|_| spine::atlas::AtlasError::Unexpected("cannot open atlas file")));
    for (n, t) in Atlas::from_reader(atlas_file).expect("cannot find atlas file").into_iter().enumerate() {

        let t = try!(t);
        let tex0 = to_tex_coords(t.xy.0,            t.xy.1);
        let tex1 = to_tex_coords(t.xy.0 + t.size.0, t.xy.1);
        let tex2 = to_tex_coords(t.xy.0 + t.size.0, t.xy.1 + t.size.1);
        let tex3 = to_tex_coords(t.xy.0,            t.xy.1 + t.size.1);
        let positions = attachments.iter().find(|&&(n, _)| n == &*t.name).map(|&(_, positions)| *positions)
                        .unwrap_or_else(|| [[0.0; 2]; 4]);
        // get 4 vertices defining rectangle texture, centered per default
        if t.rotate {
            vertices.push(Vertex { position: positions[0], tex_coords: tex3 });
            vertices.push(Vertex { position: positions[1], tex_coords: tex0 });
            vertices.push(Vertex { position: positions[2], tex_coords: tex1 });
            vertices.push(Vertex { position: positions[3], tex_coords: tex2 });
        } else {
            vertices.push(Vertex { position: positions[0], tex_coords: tex0 });
            vertices.push(Vertex { position: positions[1], tex_coords: tex1 });
            vertices.push(Vertex { position: positions[2], tex_coords: tex2 });
            vertices.push(Vertex { position: positions[3], tex_coords: tex3 });
        }

        textures.insert(t.name, 6 * n);

        let n = 4 * n as u32;
        indices.push(n);
        indices.push(n + 1);
        indices.push(n + 2);
        indices.push(n + 2);
        indices.push(n + 3);
        indices.push(n);

    }
    Ok(AtlasItems{
        textures: textures,
        vertices: vertices,
        indices: indices
    })
}


fn main() {

    // TODO: parse program arguments
    let atlas_src = "/home/johann/projects/spine-render/example/spineboy.atlas";
    let atlas_img = "/home/johann/projects/spine-render/example/spineboy.png";
    let skeleton_src = "/home/johann/projects/spine-render/example/spineboy.json";
    let skin_name = "default";
    // let anim_name = "death";
    // let anim_name = "hit";
    let anim_name = "run";
    // let anim_name = "jump";
    // let anim_name = "idle";
    let duration_ns = 1_000_000_000 / 60;

    let window = glium::glutin::WindowBuilder::new().build_glium().unwrap();

    // load texture
    let image = image::open(atlas_img).expect("couldn't read atlas image");
    let texture = glium::texture::CompressedSrgbTexture2d::new(&window, image).unwrap();

    // load skeleton from json
    let f = File::open(skeleton_src).expect("Cannot open json file");
    let skeleton = spine::skeleton::Skeleton::from_reader(BufReader::new(f)).expect("error while parsing json");
    let anim = skeleton.get_animated_skin(skin_name, Some(anim_name)).unwrap();

    // load atlas texture, indices and vertices
    let atlas = read_atlas(atlas_src,
                           texture.get_width(), texture.get_height().unwrap(),
                           &skeleton, skin_name).unwrap();

    // preload vertex buffers
    let vertex_buffer = glium::VertexBuffer::new(&window, &atlas.vertices).unwrap();
    let indice_buffer = glium::IndexBuffer::new(&window, PrimitiveType::TrianglesList, &atlas.indices).unwrap();


    // opengl programs
    let vertex_src = include_str!("../gl/spine.vert");
    let fragment_src = include_str!("../gl/spine.frag");
    let program = glium::Program::from_source(&window, vertex_src, fragment_src, None).unwrap();
    let params = glium::DrawParameters {
        blend: glium::Blend::alpha_blending(),
        .. Default::default()
    };

    // reusable perspective matrix
    let mut perspective = [[0.0; 3]; 3];

    // infinite iterator interpolating sprites every delta seconds
    let mut iter = anim.run(duration_ns as f32 / 1_000_000_000.0).cycle();

    // the main loop, supposed to run with a constant FPS
    run::start_loop(duration_ns, || {

        if let Some(sprites) = iter.next() {

            let mut target = window.draw();
            target.clear_color(0.0, 0.0, 1.0, 0.0);
            let (width, height) = target.get_dimensions();
            perspective[0][0] = 1.0 / width as f32;
            perspective[1][1] = 1.0 / height as f32;
            perspective[2][2] = 1.0;

            for sprite in sprites {
                if let Some(&n) = atlas.textures.get(sprite.attachment) {
                    let uniforms = uniform! {
                        tex: &texture,
                        perspective: perspective,
                        srt: sprite.srt.to_matrix3()
                    };
                    target.draw(&vertex_buffer,
                        indice_buffer.slice(n .. n + 6).unwrap(),
                        &program, &uniforms, &params).unwrap();
                }
            }
            target.finish().unwrap();

            for ev in window.poll_events() {
                match ev {
                    glium::glutin::Event::Closed => return run::Action::Stop,
                    _ => ()
                }
            }
        }

        run::Action::Continue
    });
}
