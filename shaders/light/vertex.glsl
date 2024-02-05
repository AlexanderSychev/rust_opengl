#version 430

layout (location = 0) in vec3 vertex_position; // Координата вершины
layout (location = 1) in vec3 vertex_normal; // Нормаль вершины

out vec3 light_intensity; // Интенсивность света

uniform vec4 light_position; // Позиция источника света
uniform vec3 kd;             // Коэффициент рассеивания
uniform vec3 ld;             // Интенсивность источника света

// Матрицы преобразований
uniform mat4 model_view_matrix;
uniform mat3 normal_matrix;
uniform mat4 projection_matrix;
uniform mat4 mvp;           // projection_matrix * model_view_matrix

void main() {
  // Преобразовать нормаль и позицию в видимые координаты
  vec3 tnorm = normalize(normal_matrix * vertex_normal);
  vec4 eye_coords = model_view_matrix * vec4(vertex_position, 1.0);

  vec3 s = normalize(vec3(light_position - eye_coords));

  // Решить уравнение рассеянного отражения
  light_intensity = ld * kd * max(dot(s, tnorm), 0.0);

  // Преобразовать позицию в усеченные координаты и передать дальше
  gl_Position = mvp * vec4(vertex_position, 1.0);
}
