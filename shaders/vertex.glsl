#version 430

in vec3 vertex_position;
in vec3 vertex_color;

out vec3 color;

uniform mat4 rotation_matrix;

void main() {
  color = vertex_color;
  gl_Position = rotation_matrix * vec4(vertex_position, 1.0);
}
