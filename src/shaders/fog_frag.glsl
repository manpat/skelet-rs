#version 130

varying vec4 v_color;
varying vec3 v_view_pos;
varying float v_emission;

void main() {
	float emission = 1.0 / v_emission - 1.0;
	// float emission = v_emission;
	emission *= emission;

	float fog_factor = length(v_view_pos) / 50.0;
	fog_factor = pow(fog_factor, 1.0/3.0);
	fog_factor = clamp(fog_factor, 0.0, 0.9);

	vec3 color = mix(v_color.xyz, vec3(0.1, 0.12, 0.11), fog_factor * (1.0 - emission));

	gl_FragColor = vec4(color, v_color.a);
}
