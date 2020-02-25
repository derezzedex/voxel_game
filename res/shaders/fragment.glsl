#version 150

in vec2 f_uv;
in vec2 f_block;

out vec4 color;
uniform sampler2DArray t;

const vec3 fogColor = vec3(0.3, 0.45, 0.65);
const float fogDensity = 0.015;

void main() {
  vec2 uv = vec2(f_uv.x, f_uv.y);
  float dist = (gl_FragCoord.z / gl_FragCoord.w) * 0.5;
  float fogFactor = 1.0 / exp( (dist * fogDensity)* (dist * fogDensity));
  fogFactor = clamp(fogFactor, 0.0, 1.0);

  vec3 textureColor = texture(t, vec3(uv, f_block.x * 16 + (15 - f_block.y))).rgb;
  color = vec4(mix(fogColor, textureColor, fogFactor), 1);
}
