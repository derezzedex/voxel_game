#version 150

in vec2 position;
in vec2 uv;

uniform mat4 p;
uniform mat4 m;

out vec2 f_uv;

void main() {
    f_uv = uv;
    gl_Position = p * m * vec4(position, 0.0, 1.0);
}
