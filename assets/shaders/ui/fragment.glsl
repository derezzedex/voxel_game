#version 150

in vec2 f_uv;

out vec4 color;
uniform sampler2D t;

void main() {
  color = texture(t, f_uv);
}
