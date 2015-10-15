#[macro_use]
extern crate glium;
extern crate image;

use std::io::Cursor;

mod atlas;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

fn vertex_from_atlas(xy: &(u16, u16), xy_tot: &(u16, u16)) -> Vertex {
    let position = [2.0 * xy.0 as f32 / xy_tot.0 as f32 - 1.0,
                    1.0 - 2.0 * xy.1 as f32 / xy_tot.1 as f32];
    let tex_coords = [xy.0 as f32 / xy_tot.0 as f32,
                      1.0 - xy.1 as f32 / xy_tot.1 as f32];
    Vertex {
        position: position,
        tex_coords: tex_coords
    }
}


fn main() {
    use glium::{DisplayBuild, Surface};

    let atlas = atlas::Atlas::load("/home/johann/projects/spine_render/spineboy.atlas");
    println!("atlas count: {}", atlas.textures.len());

    let display = glium::glutin::WindowBuilder::new().with_dimensions(1024, 256).build_glium().unwrap();
    let image = image::load(Cursor::new(&include_bytes!("../spineboy.png")[..]), image::PNG).unwrap();
    let texture = glium::texture::Texture2d::new(&display, image).unwrap();

    implement_vertex!(Vertex, position, tex_coords);

    let size_tot = (1024, 256);

    // head
    //   rotate: false
    //   xy: 279, 75
    //   size: 163, 179
    //   orig: 163, 179
    //   offset: 0, 0
    //   index: -1
    let mut shape = Vec::new();
    let mut indices = Vec::<u32>::new();
    for (_, t) in atlas.textures.iter() {

        let n = shape.len() as u32;
        let v0 = vertex_from_atlas(&(t.xy.0, t.xy.1 + t.size.1), &size_tot);
        let v1 = vertex_from_atlas(&&(t.xy.0 + t.size.0, t.xy.1 + t.size.1), &size_tot);
        let v2 = vertex_from_atlas(&(t.xy.0 + t.size.0, t.xy.1), &size_tot);
        let v3 = vertex_from_atlas(&t.xy, &size_tot);

        shape.push(v0);
        shape.push(v1);
        shape.push(v2);
        shape.push(v3);

        indices.extend([n, n+1, n+2, n+2, n+3, n].iter());

    }

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap();
    let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;
        uniform mat4 matrix;
        void main() {
            v_tex_coords = tex_coords;
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140
        in vec2 v_tex_coords;
        out vec4 color;
        uniform sampler2D tex;
        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let mut t = -0.5;

    loop {
        // we update `t`
        // t += 0.002;
        // if t > 0.5 {
        //     t = -0.5;
        // }

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 0.0);

        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ t , 0.0, 0.0, 1.0f32],
            ],
            tex: &texture,
        };

        target.draw(&vertex_buffer, &indices, &program, &uniforms,
                    &Default::default()).unwrap();
        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}
