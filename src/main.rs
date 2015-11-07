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
fn load_atlas(atlas_src: &str, width: u32, height: u32,
              attachments: &[(&str, &[[f32; 2]; 4])],
              vertices: &mut Vec<Vertex>,
              indices: &mut Vec<u32>)
    -> Result<HashMap<String, usize>, spine::atlas::AtlasError>
{
    let (width, height) = (width as f32, height as f32);

    // helper closure to convert atlas texture coordinates into vertex texture's
    let to_tex_coords = |x: u16, y: u16| {
        let (x, y) = (x as f32, y as f32);
        [x / width, 1.0 - y / height]
    };

    // iterates over atlas textures and convert it to centered rectangle (4 vertices)
    let mut textures = HashMap::new();
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
    Ok(textures)
}

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

    // load skeleton from json
    let f = File::open(skeleton_src).ok().expect("Cannot open json file");
    let reader = BufReader::new(f);
    let skeleton = spine::skeleton::Skeleton::from_reader(reader).ok().expect("error while parsing json");

    // load atlas textures
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let attachments = skeleton.get_skin("default").unwrap().attachment_positions();
    let texture_names = load_atlas(atlas_src, width, height, &attachments, &mut vertices, &mut indices).unwrap();

    // preload vertex buffers
    let vertex_buffer = glium::VertexBuffer::new(&window, &vertices).unwrap();
    let indice_buffer = glium::IndexBuffer::new(&window, PrimitiveType::TrianglesList, &indices).unwrap();

    // let anim = skeleton.get_animated_skin("default", Some("anim")).ok().expect("error while creating animated skin");
    // let anim = skeleton.get_animated_skin("default", Some("hit")).ok().expect("error while creating animated skin");
    // let anim = skeleton.get_animated_skin("default", Some("run")).ok().expect("error while creating animated skin");
    // let anim = skeleton.get_animated_skin("default", Some("death")).ok().expect("error while creating animated skin");
    let anim = skeleton.get_animated_skin("default", Some("jump")).ok().expect("error while creating animated skin");
    // let anim = skeleton.get_animated_skin("default", Some("idle")).ok().expect("error while creating animated skin");
    // let anim = skeleton.get_animated_skin("default", None).ok().expect("error while creating animated skin");


    // opengl vertex program
    let vertex_src = r#"
        #version 140

        in vec2 position;
        in vec2 tex_coords;

        out vec2 v_tex_coords;

        uniform mat3 perspective;
        uniform mat3 srt;

        void main() {
            v_tex_coords = tex_coords;
            gl_Position = vec4(perspective * srt * vec3(position, 1.0), 1.0);
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

    let mut perspective = [[0.0; 3]; 3];

    let program = glium::Program::from_source(&window, vertex_src, fragment_src, None).unwrap();
    let delta = 0.01;

    // infinite iterator interpolating sprites every delta seconds
    let mut iter = anim.run(delta).cycle();

    // the main loop, supposed to run with a constant FPS
    run::start_loop(|| {

        if let Some(sprites) = iter.next() {

            let mut target = window.draw();
            target.clear_color(0.0, 0.0, 1.0, 0.0);
            let (width, height) = target.get_dimensions();
            perspective[0][0] = 1.0 / width as f32;
            perspective[1][1] = 1.0 / height as f32;
            perspective[2][2] = 1.0;

            for sprite in sprites {
                if let Some(&n) = texture_names.get(&sprite.attachment) {
                    let srt = sprite.srt;
                    let uniforms = uniform! {
                        tex: &texture,
                        perspective: perspective,
                        srt: [
                                [ srt.cos * srt.scale[0], srt.sin, 0.0],
                                [-srt.sin, srt.cos * srt.scale[1], 0.0],
                                [ srt.position[0] , srt.position[1], 1.0f32],
                            ]
                    };
                    target.draw(&vertex_buffer,
                        indice_buffer.slice(n .. n + 6).expect(&format!("coundn't find slice {}", &n)),
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
