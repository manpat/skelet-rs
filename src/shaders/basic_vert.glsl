#version 130

uniform mat4 u_proj_view;
uniform vec4 u_color;

attribute vec3 a_vertex;

varying vec4 v_color;

void main() {
	gl_Position = u_proj_view * vec4(a_vertex, 1.0);
	v_color = u_color;
}
