#![deny(rust_2018_idioms, future_incompatible)]
#![feature(type_ascription)]

pub mod prelude;
pub mod gfx;
pub mod window;
pub mod util;

use prelude::*;
use glutin::{WindowEvent, ElementState::Pressed, MouseButton::Left as LeftMouse};
use glutin::dpi::PhysicalPosition;
use gfx::vertex::{self, ColorVertex};

fn main() -> Result<(), Box<dyn Error>> {
	let mut window = window::Window::new().expect("Failed to create window");
	let mut gfx = gfx::Gfx::new();
	// let mut mouse_pos = Vec2::zero();

	let basic_shader = gfx.core.new_shader(
		include_str!("shaders/basic_vert.glsl"),
		include_str!("shaders/frag.glsl"),
		&["a_vertex", "a_color"]
	);

	let weighted_shader = gfx.core.new_shader(
		include_str!("shaders/weighted_vert.glsl"),
		include_str!("shaders/frag.glsl"),
		&["a_vertex", "a_color", "a_bone_indices", "a_bone_weights"]
	);


	let project_data = std::fs::read("../assets/3.toy")?;
	let project = toy::load(&project_data)?;

	let bones;
	let animations;
	let model_transform;

	let mesh = gfx.core.new_mesh();
	{
		let mut mb = gfx::mesh_builder::MeshBuilder::new(mesh);

		let toy_ent = project.find_entity("Cube").expect("Missing entity");
		let toy_mesh = toy_ent.mesh_data().expect("missing mesh data");

		let animation_data = toy_mesh.animation_data.as_ref().expect("missing animation data");
		let color_data = toy_mesh.color_data(toy::DEFAULT_COLOR_DATA_NAME).expect("missing color data");

		bones = &animation_data.bones;
		animations = animation_data.animations.iter().collect(): Vec<_>;
		model_transform = toy_ent.transform();

		// println!("{:?}", toy_mesh);

		let verts = toy_mesh.positions.iter().zip(&animation_data.weights).zip(&color_data.data)
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

		mb.add_geometry(&verts, &toy_mesh.indices);

		mb.commit(&mut gfx.core);
	}

	let bone_line_mesh = gfx.core.new_mesh();
	{
		let mut mb = gfx::mesh_builder::MeshBuilder::new(bone_line_mesh);
		let color = Color::rgb(0.0, 1.0, 1.0);

		let mut verts = Vec::new();

		for (index, bone) in bones.iter().enumerate() {
			verts.push(WeightedVertex::new(bone.head, color, [index as f32, 0.0, 0.0], [1.0, 0.0, 0.0]));
			verts.push(WeightedVertex::new(bone.tail, color, [index as f32, 0.0, 0.0], [1.0, 0.0, 0.0]));
		}

		mb.add_geometry(&verts, 0..verts.len() as u16);
		mb.commit(&mut gfx.core);
	}


	let marker_mesh = gfx.core.new_mesh();
	{
		let mut mb = gfx::mesh_builder::MeshBuilder::new(marker_mesh);
		let color = Color::rgb(1.0, 1.0, 0.0);

		for toy_ent in project.entities() {
			if toy_ent.name == "Cube" { continue }

			let toy_mesh = match toy_ent.mesh_data() {
				Some(m) => m,
				None => continue
			};

			let model_transform = toy_ent.transform();

			let verts = toy_mesh.positions.iter()
				.map(move |&pos| ColorVertex::new(model_transform * pos, color.into()))
				.collect(): Vec<_>;

			mb.add_geometry(&verts, &toy_mesh.indices);
		}

		mb.commit(&mut gfx.core);
	}


	let bone_buf = gfx.core.new_texture_buffer();


	struct Instance {
		transform: Mat4,

		anim_idx: usize,
		anim_time: f32,

		bone_offset: usize,
	}

	let mut instances = vec![
		Instance {
			transform: model_transform,

			anim_idx: 0,
			anim_time: 0.0,
			bone_offset: 0,
		},
		Instance {
			transform: Mat4::translate(Vec3::new(-3.0, 0.0, 4.0)) * model_transform,

			anim_idx: 0,
			anim_time: 5.0,
			bone_offset: 0,
		},
		Instance {
			transform: Mat4::translate(Vec3::new(4.0, 0.0, 3.0)) * model_transform,

			anim_idx: 0,
			anim_time: 8.0,
			bone_offset: 0,
		},
		// Instance {
		// 	transform: Mat4::translate(Vec3::from_z(-2.0)) * model_transform,

		// 	anim_idx: 1,
		// 	anim_time: 0.0,
		// 	bone_offset: 0,
		// },
		// Instance {
		// 	transform: Mat4::translate(Vec3::new(3.0, 0.0, 2.0)) * model_transform,

		// 	anim_idx: 2,
		// 	anim_time: 0.0,
		// 	bone_offset: 0,
		// },
	];

	'main_loop: loop {
		// gfx.ui.clear_click_state();
		// elapsed += 1.0 / 60.0; 

		let events = window.poll_events();

		for event in events {
			match event {
				// WindowEvent::CursorMoved {position, ..} => {
				// 	let PhysicalPosition{x, y} = position.to_physical(window.dpi());
				// 	let pos = Vec2::new(x as f32, y as f32);
				// 	mouse_pos = window_to_screen(window.size(), pos);
				// 	gfx.ui.on_mouse_move(mouse_pos);
				// }

				// WindowEvent::MouseInput {state: Pressed, button: LeftMouse, ..} => {
				// 	// gfx.ui.on_mouse_click();
				// 	anim_idx = (anim_idx+1) % animations.len();
				// 	elapsed = 0.0;
				// 	println!("animation: {}", animations[anim_idx].name);
				// }

				WindowEvent::CloseRequested => {
					break 'main_loop
				}

				_ => {}
			}
		}

		let window_size = window.size();
		let aspect = window_size.x as f32 / window_size.y as f32;

		gfx.core.set_viewport(window_size);
		gfx.core.set_bg_color(Color::grey(0.1));
		gfx.core.clear();
		gfx.camera.update(aspect);

		// let ui_proj_view = Mat4::ortho_aspect(1.0, aspect, -100.0, 200.0);
		// let near_plane_pos = gfx.camera.inverse_projection_view() * mouse_pos.extend(0.0).extend(1.0);
		// let near_plane_pos = near_plane_pos.to_vec3() / near_plane_pos.w;

		// gfx.ui.clear();
		// gfx.ui.update(gfx.camera.forward(), near_plane_pos, aspect);


		for inst in instances.iter_mut() {
			inst.anim_time += animations[inst.anim_idx].fps/60.0;
		}


		let mut bone_frames = Vec::new();

		for Instance{anim_idx, anim_time, bone_offset, ..} in instances.iter_mut() {
			*bone_offset = bone_frames.len();

			let frames = animations[*anim_idx].channels[0].frames.len();

			let frame_0 = (*anim_time) as usize % frames;
			let frame_1 = (frame_0 + 1) % frames;

			let frame_t = anim_time.fract();

			for (channel, bone) in animations[*anim_idx].channels.iter().zip(bones.iter()) {
				let frame_0 = &channel.frames[frame_0];
				let frame_1 = &channel.frames[frame_1];

				// let position = frame_t.ease_linear(frame_0.position, frame_1.position);
				let position = frame_0.position;
				// let scale = frame_t.ease_linear(frame_0.scale, frame_1.scale);
				let scale = frame_0.scale; // frame_t.ease_linear(frame_0.scale, frame_1.scale);
				// let rotation = frame_t.ease_linear(frame_0.rotation, frame_1.rotation).normalize();
				let rotation = frame_0.rotation;

				let trans = Mat4::translate(position) // position includes offset 
					* Mat4::scale(scale)
					* rotation.to_mat4()
					* Mat4::translate(-bone.head);

				bone_frames.push(BoneFrame::from_mat4(trans));
			}
		}

		gfx.core.update_texture_buffer(bone_buf, &bone_frames);

		gfx.core.use_shader(weighted_shader);
		gfx.core.set_uniform_mat4("u_proj_view", &gfx.camera.projection_view());
		gfx.core.set_uniform_texture_buffer("u_bone_tex", bone_buf, 0);

		for Instance{bone_offset, transform, ..} in instances.iter() {
			gfx.core.set_uniform_i32("u_bone_offset", (*bone_offset) as _);
			gfx.core.set_uniform_mat4("u_object", &transform);
			gfx.core.draw_mesh(mesh);
		}

		gfx.core.use_shader(basic_shader);
		gfx.core.set_uniform_mat4("u_proj_view", &gfx.camera.projection_view());
		gfx.core.draw_mesh(marker_mesh);

		// gfx.core.use_shader(weighted_shader);
		// gfx.core.set_depth_test(false);
		// for Instance{bone_offset, transform, ..} in instances.iter() {
		// 	gfx.core.set_uniform_i32("u_bone_offset", (*bone_offset) as _);
		// 	gfx.core.set_uniform_mat4("u_object", &transform);
		// 	gfx.core.draw_mesh_lines(bone_line_mesh);
		// }
		// gfx.core.set_depth_test(true);

		window.swap();
	}

	Ok(())
}

fn window_to_screen(window_size: Vec2i, pos: Vec2) -> Vec2 {
	let window_half = window_size.to_vec2() / 2.0;
	(pos - window_half) / window_half * Vec2::new(1.0, -1.0)
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

