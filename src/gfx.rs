pub mod core;
pub mod mesh;
pub mod shader;
pub mod vertex;
pub mod mesh_builder;
pub mod camera;

use mesh_builder::MeshBuilder;

pub struct Gfx {
	pub core: core::Core,
	pub camera: camera::Camera,
}


impl Gfx {
	pub fn new() -> Gfx {
		let core = core::Core::new();

		unsafe {
			gl::Enable(gl::DEPTH_TEST);
			gl::Enable(gl::BLEND);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
		}

		Gfx {
			core,
			camera: camera::Camera::new(),
		}
	}
}