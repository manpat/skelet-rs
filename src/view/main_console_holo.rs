use crate::prelude::*;
use crate::gfx::core::Core;
use crate::gfx::mesh::MeshID;
use crate::gfx::mesh_builder::MeshBuilder;
use crate::SceneVertex;
use crate::holo_volume::HoloVolume;

pub struct MainConsoleHolo {
	mesh: MeshID<SceneVertex>,
	holo_volume: HoloVolume,
}


impl MainConsoleHolo {
	pub fn new(core: &mut Core, project: &toy::Project) -> MainConsoleHolo {
		let mesh = core.new_mesh();

		let entity = project.find_entity("HOLO_main_console")
			.expect("Couldn't find HOLO_main_console");

		let holo_volume = HoloVolume::from_entity(&entity);

		MainConsoleHolo { mesh, holo_volume }
	}

	pub fn draw(&self, core: &mut Core, ship_pos: Vec3) {
		let transform = self.holo_volume.transform;

		let mut mb = MeshBuilder::new(self.mesh);

		let right = self.holo_volume.right;
		let up = self.holo_volume.up;

		let front_center = transform * Vec3::from_z(1.0);
		let forward_scaled = transform * Vec4::from_z(-1.0);

		let left_track_bottom = transform * Vec3::new(-1.0, -1.0, 1.0);
		let right_track_bottom = transform * Vec3::new(1.0, -1.0, 1.0);
		let left_track_top = transform * Vec3::new(-1.0, 1.0, 1.0);
		let right_track_top = transform * Vec3::new(1.0, 1.0, 1.0);
		
		let move_offset = (ship_pos.z * 3.0) % 1.0;
		let move_offset = if move_offset < 0.0 { move_offset + 1.0 } else { move_offset };

		for i in 0..3 {
			let i = i as f32;
			let forward_offset = forward_scaled.to_vec3() * 50.0 * (i + move_offset);
			let color = Vec4::new(0.3, 0.8, 0.9, 1.0) * (1.0 - i/3.0);

			mb.add_quad(&[
				SceneVertex::new(left_track_bottom - right*0.1 - up*0.3 + forward_offset, color, 1.0),
				SceneVertex::new(left_track_bottom + right*0.1 - up*0.3 + forward_offset, color, 1.0),
				SceneVertex::new(left_track_bottom + right*0.1 + up*0.3 + forward_offset, color, 1.0),
				SceneVertex::new(left_track_bottom - right*0.1 + up*0.3 + forward_offset, color, 1.0),
			]);

			mb.add_quad(&[
				SceneVertex::new(right_track_bottom - right*0.1 - up*0.3 + forward_offset, color, 1.0),
				SceneVertex::new(right_track_bottom + right*0.1 - up*0.3 + forward_offset, color, 1.0),
				SceneVertex::new(right_track_bottom + right*0.1 + up*0.3 + forward_offset, color, 1.0),
				SceneVertex::new(right_track_bottom - right*0.1 + up*0.3 + forward_offset, color, 1.0),
			]);

			mb.add_quad(&[
				SceneVertex::new(left_track_top - right*0.1 - up*0.3 + forward_offset, color, 1.0),
				SceneVertex::new(left_track_top + right*0.1 - up*0.3 + forward_offset, color, 1.0),
				SceneVertex::new(left_track_top + right*0.1 + up*0.3 + forward_offset, color, 1.0),
				SceneVertex::new(left_track_top - right*0.1 + up*0.3 + forward_offset, color, 1.0),
			]);

			mb.add_quad(&[
				SceneVertex::new(right_track_top - right*0.1 - up*0.3 + forward_offset, color, 1.0),
				SceneVertex::new(right_track_top + right*0.1 - up*0.3 + forward_offset, color, 1.0),
				SceneVertex::new(right_track_top + right*0.1 + up*0.3 + forward_offset, color, 1.0),
				SceneVertex::new(right_track_top - right*0.1 + up*0.3 + forward_offset, color, 1.0),
			]);
		}

		let sub_track = transform * Vec3::new(-0.3, -0.5, 1.0);
		let move_offset = (ship_pos.z * 30.0) % 1.0;
		let move_offset = if move_offset < 0.0 { move_offset + 1.0 } else { move_offset };

		for i in 0..40 {
			let i = i as f32;
			let forward_offset = forward_scaled.to_vec3() * 0.5 * (i + move_offset);
			let color = Vec4::new(0.9, 0.7, 0.2, 1.0) * (1.0 - i.min(25.0)/30.0);

			mb.add_quad(&[
				SceneVertex::new(sub_track - right*0.1 - up*0.1 + forward_offset, color, 1.0),
				SceneVertex::new(sub_track + right*0.1 - up*0.1 + forward_offset, color, 1.0),
				SceneVertex::new(sub_track + right*0.1 + up*0.1 + forward_offset, color, 1.0),
				SceneVertex::new(sub_track - right*0.1 + up*0.1 + forward_offset, color, 1.0),
			]);
		}

		mb.commit(core);


		core.draw_mesh(self.mesh);
	}
}