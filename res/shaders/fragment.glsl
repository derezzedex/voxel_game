#version 150

in vec2 f_uv;
in vec2 f_block;

out vec4 color;
uniform sampler2DArray t;

void main() {
  vec2 uv = vec2(f_uv.x, f_uv.y);
  color = texture(t, vec3(uv, f_block.x * 16 + (15 - f_block.y)));
  if (color.a < 0.5){
    discard;
  }
}
