use crate::prelude::*;

use super::core::Core;
use super::shader::ShaderID;
use super::vertex::ColorVertex;
use super::mesh::BasicMeshID;
use super::camera::Camera;


pub struct Debug {
	shader: ShaderID,

	points: Vec<ColorVertex>,
	lines: Vec<ColorVertex>,

	points_mesh: BasicMeshID<ColorVertex>,
	lines_mesh: BasicMeshID<ColorVertex>,
}

impl Debug {
	pub fn new(core: &mut Core) -> Debug {
		let shader = core.new_shader(
			include_str!("../shaders/color_vert.glsl"),
			include_str!("../shaders/color_frag.glsl"),
			&["a_vertex", "a_color"]
		);

		let points_mesh = core.new_basic_mesh();
		let lines_mesh = core.new_basic_mesh();

		Debug {
			shader,

			points: Vec::new(),
			lines: Vec::new(),

			points_mesh,
			lines_mesh,
		}
	}

	pub fn draw(&mut self, core: &mut Core, camera: &Camera) {
		core.set_depth(None);

		core.use_shader(self.shader);
		core.set_uniform_mat4("u_proj_view", &camera.projection_view());

		if !self.points.is_empty() {
			core.update_basic_mesh(self.points_mesh, &self.points);
			core.draw_mesh_points(self.points_mesh);
			self.points.clear();
		}

		if !self.lines.is_empty() {
			core.update_basic_mesh(self.lines_mesh, &self.lines);
			core.draw_mesh_lines(self.lines_mesh);
			self.lines.clear();
		}

		core.set_depth(super::core::DepthFunc::default());
	}

	pub fn point(&mut self, world: Vec3, color: Color) {
		self.points.push(ColorVertex::new(world, color.into()));
	}

	pub fn line(&mut self, start: Vec3, end: Vec3, color: Color) {
		let color = color.into();
		self.lines.push(ColorVertex::new(start, color));
		self.lines.push(ColorVertex::new(end, color));
	}
}