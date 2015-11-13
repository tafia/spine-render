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
