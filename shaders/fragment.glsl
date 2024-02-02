#version 430

in vec3 tex_coord;

out vec4 frag_color;

layout (binding = 0, std140) uniform blob_settings {
  vec4 inner_color;
  vec4 outer_color;
  float radius_inner;
  float radius_outer;
};

void main() {
  float dx = tex_coord.x - 0.5;
  float dy = tex_coord.y - 0.5;
  float dist = sqrt(dx * dx + dy * dy);

  frag_color = mix(inner_color, outer_color, smoothstep(radius_inner, radius_outer, dist));
}
