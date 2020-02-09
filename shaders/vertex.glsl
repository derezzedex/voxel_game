#version 150

in vec3 position;
in vec3 color;
in vec3 normal;

uniform mat4 m;
uniform mat4 v;
uniform mat4 p;

out vec2 uv_coords;
out vec2 bpos;

void main() {
    uv_coords = color.xy;
    bpos = normal.xy;
    gl_Position = p * v * m * vec4(position, 1.0);
}
