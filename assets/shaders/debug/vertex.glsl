#version 150

in vec3 position;
in vec4 color;

uniform mat4 m;
uniform mat4 p;
uniform mat4 v;

out vec4 f_color;

void main() {
    f_color = color;
    gl_Position = p * m * v * vec4(position, 1.0);
}
