use crate::prelude::*;
use crate::gfx::camera::{self, Camera};
use crate::gfx::core::Core;
use crate::gfx::mesh::MeshID;
use crate::holo_volume::HoloVolume;
use crate::SceneVertex;

pub struct Lab {
	left_holo_mesh: MeshID<SceneVertex>,
	right_holo_mesh: MeshID<SceneVertex>,

	left_holo_volume: HoloVolume,
	right_holo_volume: HoloVolume,
}


impl Lab {
	pub fn new(core: &mut Core, project: &toy::Project) -> Lab {
		let left_entity = project.find_entity("HOLO_lab_console_left")
			.expect("Couldn't find HOLO_lab_console_left");

		let right_entity = project.find_entity("HOLO_lab_console_right")
			.expect("Couldn't find HOLO_lab_console_right");

		let left_holo_volume = HoloVolume::from_entity(&left_entity);
		let right_holo_volume = HoloVolume::from_entity(&right_entity);

		let left_holo_mesh = core.new_mesh();
		let right_holo_mesh = core.new_mesh();

		Lab {
			left_holo_mesh, right_holo_mesh,
			left_holo_volume, right_holo_volume,
		}
	}

	pub fn draw(&mut self, core: &mut Core) {
		core.draw_mesh(self.left_holo_mesh);
		core.draw_mesh(self.right_holo_mesh);
	}
}