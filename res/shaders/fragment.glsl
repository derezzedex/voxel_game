#version 150

in vec2 f_uv;
in vec2 f_block;
in vec4 f_viewspace;

out vec4 color;
uniform sampler2DArray t;

const vec3 fog_color = vec3(0.3, 0.45, 0.65);
const float fog_density = 0.008;
const float fog_gradient = 5.0;

void main() {
  vec3 texture_color = texture(t, vec3(f_uv, f_block.x * 16 + (15 - f_block.y))).rgb;

  float dist = length(f_viewspace);
  float visibility = exp(-pow((dist * fog_density), fog_gradient));
  visibility = clamp(visibility, 0.0, 1.0);

  color = vec4(mix(fog_color, texture_color, visibility), 1);
}
