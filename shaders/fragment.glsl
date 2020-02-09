#version 150

in vec2 uv_coords;
in vec2 bpos;
out vec4 color;

uniform sampler2DArray t;

void main() {
  vec2 uv = vec2(uv_coords.x, uv_coords.y);
  color = texture(t, vec3(uv, bpos.x * 16 + (15 - bpos.y)));
}
