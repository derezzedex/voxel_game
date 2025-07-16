#version 150

in vec2 f_uv;
in vec2 f_block;
in vec4 f_viewspace;
in vec4 f_tint;

out vec4 color;
uniform sampler2DArray t;

const vec4 fog_color = vec4(0.3, 0.45, 0.65, 1.0);
const float fog_density = 0.02;
const float fog_gradient = 16.;

void main() {
  vec4 tex = texture(t, vec3(f_uv, f_block.x * 16 + (15 - f_block.y)));
  tex = vec4(tex.rgb * f_tint.rgb, tex.a);
  float dist = length(f_viewspace) * 0.75;
  float visibility = exp(-pow((dist * fog_density), fog_gradient));
  visibility = clamp(visibility, 0.0, 1.0);
  color = mix(fog_color, tex, visibility);
}
