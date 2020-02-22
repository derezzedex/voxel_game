#version 150

in vec3 position;
in vec2 uv;
in vec2 block;

uniform mat4 m;
uniform mat4 v;
uniform mat4 p;

out vec2 f_uv;
out vec2 f_block;

void main() {
    f_uv = uv;
    f_block = block;
    gl_Position = p * v * m * vec4(position, 1.0);
}
