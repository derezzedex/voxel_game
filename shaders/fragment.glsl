#version 150

in vec2 tex_coords;
in vec3 colors;

out vec4 color;

void main() {
    color = vec4(colors, 1.0);

}
