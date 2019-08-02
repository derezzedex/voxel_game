#version 150

in vec3 position;
in vec3 color;
in vec2 tex_coord;

uniform mat4 m;
uniform mat4 v;
uniform mat4 p;

out vec2 tex_coords;
out vec3 colors;

void main() {
    tex_coords = tex_coord;
    gl_Position = p * v * m * vec4(position, 1.0);
    colors = color;
}
