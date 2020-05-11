#version 150

in vec3 position;
in vec2 uv;
in vec2 block;
in vec4 tint;

uniform mat4 m;
uniform mat4 v;
uniform mat4 p;

out vec2 f_uv;
out vec2 f_block;
out vec4 f_viewspace;
out vec4 f_tint;

void main() {
    f_uv = uv;
    f_block = block;
    f_viewspace = v * m * vec4(position, 1);
    f_tint = tint;
    gl_Position = p * v * m * vec4(position, 1.0);
}
