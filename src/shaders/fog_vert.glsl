#version 130

uniform mat4 u_proj_view;
uniform mat4 u_view;

attribute vec3 a_vertex;
attribute vec4 a_color;
attribute float a_emission;

varying vec4 v_color;
varying vec3 v_view_pos;
varying float v_emission;

void main() {
	gl_Position = u_proj_view * vec4(a_vertex, 1.0);
	v_color = a_color;
	v_emission = 1.0 / (1.0 + a_emission);
	// v_emission = a_emission;

	v_view_pos = (u_view * vec4(a_vertex, 1.0)).xyz;
}
