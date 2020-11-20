#version 450
//
// layout(location=0) in vec2 v_tex_coords;
layout(location=0) out vec4 color;
//
// layout(set=0, binding=0) uniform texture2D t_tex;
// layout(set=0, binding=1) uniform sampler s_tex;
//
// void main(){
//   color = texture(sampler2D(t_tex, s_tex), v_tex_coords);
// }

/* sum(rgb * a, a) */
layout(set=0, binding=0) uniform sampler2D accumTexture;

/* prod(1 - a) */
layout(set=0, binding=1) uniform sampler2D revealageTexture;

float maxComponent (vec4 v) {
  return max(max(max(v.x, v.y), v.z), v.w);
}

void main() {
    ivec2 coord = ivec2(gl_FragCoord.xy);
    float revealage = texelFetch(revealageTexture, coord, 0).r;
    vec4 accum = texelFetch(accumTexture, coord, 0);

    // Suppress overflow
    if (isinf(maxComponent(abs(accum)))) {
        accum.rgb = vec3(accum.a);
    }

    vec3 averageColor = accum.rgb / max(accum.a, 0.00001);

    // color =  (accum.rgb / accum.a) * (1 - revealage) + dst * revealage
    color = vec4(averageColor, 1. - revealage);
    // color = vec4(revealage, 0, 0, 1);
}
