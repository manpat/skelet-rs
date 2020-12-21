use crate::prelude::*;
use crate::gfx::camera::{self, Camera};
use crate::gfx::core::Core;
use crate::gfx::vertex::BasicVertex;
use crate::gfx::mesh::MeshID;
use crate::gfx::shader::ShaderID;

use super::main_console_holo::MainConsoleHolo;

pub struct ViewScreen {
	camera: Camera,

	screen_mesh: MeshID<BasicVertex>,
	screen_shader: ShaderID,

	scene_mesh: MeshID<crate::SceneVertex>,
	scene_shader: ShaderID,

	fullscreen_mesh: MeshID<BasicVertex>,

	main_console_holo: MainConsoleHolo,
}

impl ViewScreen {
	pub fn new(core: &mut Core, project: &toy::Project) -> ViewScreen {
		let mut camera = Camera::new(
			camera::ProjectionMode::Perspective { fov_y: PI/3.0 },
			camera::ViewMode::FirstPerson
		);

		camera.set_near_far(0.1, 1000.0);
		camera.set_position(Vec3::from_z(50.0));

		let entity = project.find_entity("SCR_viewing_screen")
			.expect("Can't find viewing screen!");

		let mesh_data = entity.mesh_data().expect("Viewing screen missing mesh");
		let transform = entity.transform();

		let screen_verts = mesh_data.positions.iter()
			.map(|&v| BasicVertex(transform * v))
			.collect(): Vec<_>;

		let screen_mesh = core.new_mesh();
		core.update_mesh(screen_mesh, &screen_verts, &mesh_data.indices);

		let screen_shader = core.new_shader(
			include_str!("../shaders/basic_vert.glsl"),
			include_str!("../shaders/color_frag.glsl"),
			&["a_vertex"]
		);


		let scene = project.find_scene("space")
			.expect("Can't find space scene");

		let scene_mesh = crate::build_scene_mesh(core, scene);
		let scene_shader = core.new_shader(
			include_str!("../shaders/fog_vert.glsl"),
			include_str!("../shaders/fog_frag.glsl"),
			&["a_vertex", "a_color", "a_emission"]
		);


		let fullscreen_verts = [
			BasicVertex(Vec3::new(-1.0, -1.0, 1.0)),
			BasicVertex(Vec3::new( 1.0, -1.0, 1.0)),
			BasicVertex(Vec3::new( 1.0,  1.0, 1.0)),
			BasicVertex(Vec3::new(-1.0,  1.0, 1.0)),
		];

		let fullscreen_mesh = core.new_mesh();
		core.update_mesh(fullscreen_mesh, &fullscreen_verts, &[0, 1, 2, 0, 2, 3]);

		ViewScreen {
			camera,
			screen_mesh,
			screen_shader,

			scene_mesh,
			scene_shader,

			fullscreen_mesh,

			main_console_holo: MainConsoleHolo::new(core, project),
		}
	}


	pub fn draw(&mut self, core: &mut Core, ply_camera: &Camera) {
		use crate::gfx::core::StencilParams;

		self.camera.update(ply_camera.viewport());
		self.camera.set_yaw(ply_camera.yaw());
		self.camera.set_pitch(ply_camera.pitch());

		let new_position = self.camera.position() + Vec3::from_z(-0.4 / 60.0);
		self.camera.set_position(new_position);

		// Draw view screen into stencil
		core.use_shader(self.screen_shader);
		core.set_uniform_mat4("u_proj_view", &ply_camera.projection_view());
		core.set_uniform_vec4("u_color", Vec4::splat(1.0));
		
		core.set_stencil(StencilParams::write_on_depth_pass(1));
		core.set_color_write(false);
		core.draw_mesh(self.screen_mesh);

		// Clear depth where stencil
		self.clear_depth_stenciled(core, true);

		// Draw space
		core.use_shader(self.scene_shader);

		core.set_uniform_mat4("u_proj_view", &(self.camera.projection_view()));
		core.set_uniform_mat4("u_view", &self.camera.view_matrix());
		core.draw_mesh(self.scene_mesh);

		// Clear depth where stencil
		self.clear_depth_stenciled(core, false);

		// Draw hologram
		core.use_shader(self.scene_shader);
		core.set_uniform_mat4("u_proj_view", &ply_camera.projection_view());
		core.set_uniform_mat4("u_view", &ply_camera.view_matrix());
		self.main_console_holo.draw(core, self.camera.position());

		core.set_stencil(None);
	}

	fn clear_depth_stenciled(&self, core: &mut Core, color_fill: bool) {
		use crate::gfx::core::{StencilParams, DepthFunc};

		let space_color = Vec4::splat(0.1);

		core.use_shader(self.screen_shader);
		core.set_uniform_mat4("u_proj_view", &Mat4::ident());
		core.set_uniform_vec4("u_color", space_color);

		core.set_stencil(StencilParams::stencil_equal(1));
		core.set_depth(DepthFunc::Always);
		core.set_color_write(color_fill);
		core.draw_mesh(self.fullscreen_mesh);

		core.set_color_write(true);
		core.set_depth(DepthFunc::default());
	}
}