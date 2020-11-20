#version 450

layout(location=0) in vec2 v_tex_coord;

layout(set=0, binding=0) uniform texture2D t_tex;
layout(set=0, binding=1) uniform sampler s_tex;

layout(location=0) out vec4 _accum;
layout(location=1) out float _revealage;

void main(){
  vec4 color = texture(sampler2D(t_tex, s_tex), v_tex_coord);

  float weight =
      max(min(1.0, max(max(color.r, color.g), color.b) * color.a), color.a) *
      clamp(0.03 / (1e-5 + pow(gl_FragCoord.z / 200, 4.0)), 1e-2, 3e3);
  _accum = vec4(color.rgb * color.a, color.a) * weight;
  _revealage = color.a;
}
