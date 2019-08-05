#version 150

in vec3 position;
in vec3 color;

uniform mat4 m;
uniform mat4 v;
uniform mat4 p;

out vec3 frag_color;

void main() {
    frag_color = color;
    gl_Position = p * v * m * vec4(position, 1.0);
}
