#version 140

uniform mat4 u_proj_view;
uniform samplerBuffer u_bone_tex;

attribute vec3 a_vertex;
attribute vec4 a_color;
attribute vec3 a_bone_indices;
attribute vec3 a_bone_weights;

varying vec4 v_color;


mat4x3 read_bone(in float index) {
	vec4 row_0 = texelFetch(u_bone_tex, int(index)*3+0);
	vec4 row_1 = texelFetch(u_bone_tex, int(index)*3+1);
	vec4 row_2 = texelFetch(u_bone_tex, int(index)*3+2);

	mat3x4 transposed;
	transposed[0] = row_0;
	transposed[1] = row_1;
	transposed[2] = row_2;
	return transpose(transposed);
}

void main() {
	mat4x3 bone_0 = read_bone(a_bone_indices.x);
	mat4x3 bone_1 = read_bone(a_bone_indices.y);
	mat4x3 bone_2 = read_bone(a_bone_indices.z);

	float resting_weight = 1.0 - (a_bone_weights.x + a_bone_weights.y + a_bone_weights.z);
	vec3 vert_rest = a_vertex * resting_weight;
	vec3 vert_0 = bone_0 * vec4(a_vertex, 1.0) * a_bone_weights.x;
	vec3 vert_1 = bone_1 * vec4(a_vertex, 1.0) * a_bone_weights.y;
	vec3 vert_2 = bone_2 * vec4(a_vertex, 1.0) * a_bone_weights.z;

	vec3 final_vert = vert_rest + vert_0 + vert_1 + vert_2;

	gl_Position = u_proj_view * vec4(final_vert, 1.0);
	v_color = a_color; // vec4(a_bone_indices/4.0, 1.0);
}
