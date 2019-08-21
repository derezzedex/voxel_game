#version 150

in vec3 frag_color;
out vec4 color;

uniform sampler2D t;

void main() {
    vec2 tex_coords = frag_color.xy;
    color = texture(t, tex_coords);
}
