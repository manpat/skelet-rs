use crate::prelude::*;
use crate::gfx;

use super::core::Core;
use super::camera::Camera;
use super::shader::ShaderID;
use super::mesh::{MeshID, BasicMeshID};
use super::texture_buffer::TextureBufferID;
use super::vertex;

use std::collections::HashMap;

pub type AnimatedMeshID = MeshID<WeightedVertex>;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct AnimationID(AnimatedMeshID, usize);


pub struct AnimatedMeshInstance {
	pub transform: Mat4, // TODO: Mat4x3
	pub animation: AnimationID,

	/// animation progress in frames
	// TODO: should this just be internal only?
	pub animation_time: f32,
}

struct AnimatedMeshData {
	animations: Vec<Animation>,
	bones: Vec<(Vec3, Vec3)>,

	bone_mesh: BasicMeshID<WeightedVertex>,
}

pub struct Animation {
	pub name: String,
	pub fps: f32,

	/// the packed animation for all frames for all bones - laid out linearly.
	/// a sequence of `bone_count` elements represents a single frame
	frame_data: Vec<BoneFrame>,
	pub bone_count: usize,
	pub frame_count: usize,
}

pub struct AnimationManager {
	bone_buffer: TextureBufferID<BoneFrame>,
	instances: Vec<AnimatedMeshInstance>,
	shader: ShaderID,

	mesh_animations: HashMap<AnimatedMeshID, AnimatedMeshData>,

	draw_bone_debug: bool,
}


impl AnimationManager {
	pub fn new(core: &mut Core) -> AnimationManager {
		let bone_buffer = core.new_texture_buffer();

		let shader = core.new_shader(
			include_str!("../shaders/weighted_vert.glsl"),
			include_str!("../shaders/frag.glsl"),
			&["a_vertex", "a_color", "a_bone_indices", "a_bone_weights"]
		);

		AnimationManager {
			bone_buffer,
			instances: Vec::new(),
			shader,

			mesh_animations: HashMap::new(),
			draw_bone_debug: false,
		}
	}

	pub fn register_animated_mesh(&mut self, core: &mut Core, mesh_data: &toy::MeshData) -> AnimatedMeshID {
		let mesh = core.new_mesh();
		let mut mb = gfx::mesh_builder::MeshBuilder::new(mesh);

		let animation_data = mesh_data.animation_data.as_ref().expect("missing animation data");
		let color_data = mesh_data.color_data(toy::DEFAULT_COLOR_DATA_NAME).expect("missing color data");
		// TODO: color_data should be optional

		let verts = mesh_data.positions.iter().zip(&animation_data.weights).zip(&color_data.data)
			.map(move |((&pos, vertex_weight), &color)| {
				let toy::MeshWeightVertex{indices, weights} = *vertex_weight;

				let indices = [
					indices[0] as f32,
					indices[1] as f32,
					indices[2] as f32,
				];

				WeightedVertex::new(pos, color.into(), indices, weights)
			})
			.collect(): Vec<_>;

		mb.add_geometry(&verts, &mesh_data.indices);
		mb.commit(core);

		let mut animations = Vec::with_capacity(animation_data.animations.len());

		for anim in animation_data.animations.iter() {
			assert!(anim.channels.len() > 0);

			let frame_count = anim.channels[0].frames.len();
			let bone_count = anim.channels.len();

			assert!(bone_count == animation_data.bones.len());
			for channel in anim.channels.iter() {
				assert!(channel.frames.len() == frame_count);
			}

			let mut frame_data = Vec::with_capacity(bone_count * frame_count);
			for frame_idx in 0..frame_count {
				for (channel, bone) in anim.channels.iter().zip(&animation_data.bones) {
					let frame = &channel.frames[frame_idx];

					// frame.position includes bone.head - which is why theres no need to unoffset again
					let trans = Mat4::translate(frame.position) 
						* Mat4::scale(frame.scale)
						* frame.rotation.to_mat4()
						* Mat4::translate(-bone.head);

					frame_data.push(BoneFrame::from_mat4(trans));
				}
			}

			animations.push(Animation {
				name: anim.name.clone(),
				fps: anim.fps,

				frame_data,
				frame_count,
				bone_count,
			});
		}

		let bone_mesh = core.new_basic_mesh();
		{
			let bone_color = Color::rgb(0.5, 0.1, 0.5);

			let to_vert = |b, idx| WeightedVertex::new(b, bone_color, [0.0, idx, 0.0], [0.0, 1.0, 0.0]);
			let verts = animation_data.bones.iter().enumerate()
				.flat_map(move |(idx, b)| {
					let v0 = std::iter::once(to_vert(b.head, idx as f32));
					let v1 = std::iter::once(to_vert(b.tail, idx as f32));
					v0.chain(v1)
				})
				.collect(): Vec<_>;

			core.update_basic_mesh(bone_mesh, &verts);
		}

		let bones = animation_data.bones.iter()
			.map(|bone| (bone.head, bone.tail))
			.collect();

		self.mesh_animations.insert(mesh, AnimatedMeshData{animations, bones, bone_mesh});

		mesh
	}


	pub fn add_instance(&mut self, instance: AnimatedMeshInstance) {
		self.instances.push(instance);
	}


	pub fn animations_for(&self, mesh_id: AnimatedMeshID) -> impl Iterator<Item=(AnimationID, &Animation)> {
		self.mesh_animations.get(&mesh_id)
			.expect("trying to get animations for unregistered mesh")
			.animations.iter()
			.enumerate()
			.map(move |(idx, anim)| (AnimationID(mesh_id, idx), anim))
	}

	pub fn animation_by_name<'s>(&'s self, mesh_id: AnimatedMeshID, name: &str) -> Option<(AnimationID, &'s Animation)> {
		self.animations_for(mesh_id)
			.find(|(_, anim)| anim.name == name)
	}

	pub fn animation_meta(&self, AnimationID(mesh_id, anim_idx): AnimationID) -> Option<&Animation> {
		self.mesh_animations.get(&mesh_id)
			.and_then(|d| d.animations.get(anim_idx))
	}


	pub fn clear(&mut self) {
		self.instances.clear();
	}

	pub fn draw(&self, core: &mut Core, camera: &Camera) {
		let mut bone_frames = Vec::new();
		let mut bone_offsets = Vec::new();

		for AnimatedMeshInstance{animation, animation_time, ..} in self.instances.iter() {
			bone_offsets.push(bone_frames.len());

			let AnimationID(mesh, idx) = animation;
			let animated_mesh_data = self.mesh_animations.get(mesh)
				.expect("trying to get animation data for unregistered mesh");

			let animation = animated_mesh_data.animations.get(*idx)
				.expect("trying to get non-existent animation for mesh");

			let frame_number = (*animation_time) as usize % animation.frame_count;
			let frame_data_start = animation.bone_count * frame_number;
			let frame_data_end = frame_data_start + animation.bone_count;

			let frame_data = &animation.frame_data[frame_data_start..frame_data_end];
			bone_frames.extend_from_slice(frame_data);
		}

		core.update_texture_buffer(self.bone_buffer, &bone_frames);

		core.use_shader(self.shader);
		core.set_uniform_mat4("u_proj_view", &camera.projection_view());
		core.set_uniform_texture_buffer("u_bone_tex", self.bone_buffer, 0);

		for (inst, bone_offset) in self.instances.iter().zip(&bone_offsets) {
			let AnimationID(mesh, _) = inst.animation;

			core.set_uniform_i32("u_bone_offset", *bone_offset as _);
			core.set_uniform_mat4("u_object", &inst.transform);
			core.draw_mesh(mesh);
		}

		// debug bone viz
		if self.draw_bone_debug {
			core.set_depth_test(false);
			for (AnimatedMeshInstance{transform, animation, ..}, bone_offset) in self.instances.iter().zip(&bone_offsets) {
				let AnimationID(mesh, _) = animation;
				let animated_mesh_data = self.mesh_animations.get(mesh)
					.expect("trying to get animation data for unregistered mesh");

				core.set_uniform_i32("u_bone_offset", *bone_offset as _);
				core.set_uniform_mat4("u_object", &transform);
				core.draw_mesh_lines(animated_mesh_data.bone_mesh);
			}
			core.set_depth_test(true);
		}
	}
}





#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct BoneFrame {
	rows: [Vec4; 3],
}

impl BoneFrame {
	fn from_mat4(m: Mat4) -> BoneFrame {
		let mut rows = [Vec4::zero(); 3];
		rows.copy_from_slice(&m.rows[..3]);
		BoneFrame {rows}
	}
}



const WEIGHTS_PER_VERTEX: usize = 3;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WeightedVertex {
	pub pos: Vec3,
	pub color: Vec4,
	pub bone_indices: [f32; WEIGHTS_PER_VERTEX],
	pub bone_weights: [f32; WEIGHTS_PER_VERTEX],
}

impl WeightedVertex {
	pub fn new(pos: Vec3, color: Color, bone_indices: [f32; WEIGHTS_PER_VERTEX],
		bone_weights: [f32; WEIGHTS_PER_VERTEX]) -> Self
	{
		WeightedVertex{pos, color: color.into(), bone_indices, bone_weights}
	}
}

impl vertex::Vertex for WeightedVertex {
	fn descriptor() -> vertex::Descriptor {
		vertex::Descriptor::from(&[3, 4, WEIGHTS_PER_VERTEX as _, WEIGHTS_PER_VERTEX as _])
	}
}

