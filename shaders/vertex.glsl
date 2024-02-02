#version 430

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_tex_coord;

out vec3 tex_coord;

void main() {
  tex_coord = vertex_tex_coord;
  gl_Position = vec4(vertex_position, 1.0);
}
