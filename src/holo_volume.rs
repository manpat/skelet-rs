use crate::prelude::*;

pub struct HoloVolume {
	pub transform: Mat4,
	pub forward: Vec3,
	pub right: Vec3,
	pub up: Vec3,
}


impl HoloVolume {
	pub fn from_entity(entity: &toy::EntityData) -> HoloVolume {
		let transform = entity.transform();
		let forward = entity.rotation.forward();
		let right = entity.rotation.right();
		let up = entity.rotation.up();

		HoloVolume {
			transform,
			forward,
			right,
			up,
		}
	}
}