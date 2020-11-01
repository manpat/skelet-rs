#![deny(rust_2018_idioms, future_incompatible)]
#![feature(type_ascription)]

pub mod prelude;
pub mod gfx;
pub mod window;
pub mod util;

use prelude::*;
use glutin::{WindowEvent, ElementState::Pressed, MouseButton::Left as LeftMouse};
use glutin::dpi::PhysicalPosition;
use gfx::vertex;

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


	let project_data = std::fs::read("../assets/main.toy")?;
	let project = toy::load(&project_data)?;

	let mesh = gfx.core.new_mesh();
	{
		let mut mb = gfx::mesh_builder::MeshBuilder::new(mesh);
		let color = Color::rgb(1.0, 0.0, 0.0);

		// mb.add_quad(&[
		// 	WeightedVertex::new(Vec3::new(-0.2 - 0.5, -0.2, 0.0), color, [1.0, 2.0], [0.5, 0.0]),
		// 	WeightedVertex::new(Vec3::new(-0.2 - 0.5,  0.2, 0.0), color, [1.0, 2.0], [0.0, 0.0]),
		// 	WeightedVertex::new(Vec3::new( 0.2 - 0.5,  0.2, 0.0), color, [1.0, 2.0], [0.7, 0.0]),
		// 	WeightedVertex::new(Vec3::new( 0.2 - 0.5, -0.2, 0.0), color, [1.0, 2.0], [1.0, 0.0]),
		// ]);

		// mb.add_quad(&[
		// 	WeightedVertex::new(Vec3::new(-0.2, -0.2, 0.0), color, [1.0, 3.0], [0.5, 0.5]),
		// 	WeightedVertex::new(Vec3::new(-0.2,  0.2, 0.0), color, [1.0, 2.0], [0.9, 0.1]),
		// 	WeightedVertex::new(Vec3::new( 0.2,  0.2, 0.0), color, [1.0, 2.0], [0.1, 0.9]),
		// 	WeightedVertex::new(Vec3::new( 0.2, -0.2, 0.0), color, [2.0, 3.0], [0.3, 0.7]),
		// ]);

		// mb.add_quad(&[
		// 	WeightedVertex::new(Vec3::new(-0.2 + 0.5, -0.2, 0.0), color, [1.0, 2.0], [0.0, 1.0]),
		// 	WeightedVertex::new(Vec3::new(-0.2 + 0.5,  0.2, 0.0), color, [1.0, 2.0], [0.0, 1.0]),
		// 	WeightedVertex::new(Vec3::new( 0.2 + 0.5,  0.2, 0.0), color, [1.0, 2.0], [0.0, 1.0]),
		// 	WeightedVertex::new(Vec3::new( 0.2 + 0.5, -0.2, 0.0), color, [1.0, 2.0], [0.0, 1.0]),
		// ]);

		// mb.add_quad(&[
		// 	WeightedVertex::new(Vec3::new(-0.4, -0.2-0.5, 0.0), color, [3.0, 2.0], [1.0, 0.0]),
		// 	WeightedVertex::new(Vec3::new(-0.4,  0.2-0.5, 0.0), color, [3.0, 2.0], [1.0, 0.0]),
		// 	WeightedVertex::new(Vec3::new( 0.4,  0.2-0.5, 0.0), color, [3.0, 2.0], [0.3, 0.7]),
		// 	WeightedVertex::new(Vec3::new( 0.4, -0.2-0.5, 0.0), color, [3.0, 2.0], [1.0, 0.0]),
		// ]);

		let toy_ent = project.find_entity("Cube").expect("Missing entity 'Cube'");
		let toy_mesh = toy_ent.mesh_data().expect("'Cube' missing mesh data");

		let transform = Mat4::translate(toy_ent.position)
			* toy_ent.rotation.to_mat4()
			* Mat4::scale(toy_ent.scale);

		println!("{:?}", toy_ent);
		println!("{:?}", toy_mesh);

		let verts = toy_mesh.positions.iter()
			.map(move |&v| vertex::ColorVertex::new(transform * v, color.into()) )
			.collect(): Vec<_>;

		mb.add_geometry(&verts, &toy_mesh.indices);

		mb.commit(&mut gfx.core);
	}

	let (bone_tex_id, bone_buf_id) = generate_bone_texture();
	let mut elapsed = 0.0f32;

	'main_loop: loop {
		// gfx.ui.clear_click_state();
		elapsed += 1.0 / 60.0; 

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
				// 	gfx.ui.on_mouse_click();
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

		let trans_0 = Mat4::translate(Vec3::from_x(-0.5)) * Mat4::zrot((elapsed*2.1).sin() * 0.3) * Mat4::translate(Vec3::from_x(0.5));
		let trans_1 = Mat4::translate(Vec3::from_y(-0.5)) * Mat4::yrot((elapsed*1.6).cos() * 0.5) * Mat4::translate(Vec3::from_y(0.5));

		let bones = [
			Bone::from_mat4(Mat4::ident()),
			Bone::from_mat4(trans_0),
			Bone::from_mat4(Mat4::translate(Vec3::new(0.0, 0.2 + 0.5 * (elapsed*1.2).sin(), 0.0))),
			Bone::from_mat4(trans_1),
		];

		update_bone_texture(bone_buf_id, &bones);

		gfx.core.use_shader(weighted_shader);

		unsafe {
			gl::BindTexture(gl::TEXTURE_BUFFER, bone_tex_id);
			gfx.core.set_uniform_i32("u_bone_tex", 0);
		}

		gfx.core.set_uniform_mat4("u_proj_view", &gfx.camera.projection_view());
		gfx.core.draw_mesh(mesh);

		window.swap();
	}

	Ok(())
}

fn window_to_screen(window_size: Vec2i, pos: Vec2) -> Vec2 {
	let window_half = window_size.to_vec2() / 2.0;
	(pos - window_half) / window_half * Vec2::new(1.0, -1.0)
}




#[repr(C)]
struct Bone {
	rows: [Vec4; 3],
}

impl Bone {
	fn from_mat4(m: Mat4) -> Bone {
		let mut rows = [Vec4::zero(); 3];
		rows.copy_from_slice(&m.rows[..3]);
		Bone {rows}
	}
}

fn generate_bone_texture() -> (u32, u32) {
	let (mut tex_id, mut buf_id) = (0, 0);
	unsafe {
		gl::GenTextures(1, &mut tex_id);
		gl::GenBuffers(1, &mut buf_id);

		gl::BindBuffer(gl::TEXTURE_BUFFER, buf_id);
		gl::BindTexture(gl::TEXTURE_BUFFER, tex_id);
		gl::TexBuffer(gl::TEXTURE_BUFFER, gl::RGBA32F, buf_id);
	}


	(tex_id, buf_id)
}

fn update_bone_texture(buf_id: u32, bones: &[Bone]) {
	let buffer_size = bones.len() * std::mem::size_of::<Bone>();
	assert!(buffer_size < 65536); // GL_MAX_TEXTURE_BUFFER_SIZE

	unsafe {
		gl::BindBuffer(gl::TEXTURE_BUFFER, buf_id);
		gl::BufferData(
			gl::TEXTURE_BUFFER,
			buffer_size as _,
			bones.as_ptr() as _,
			gl::STREAM_DRAW
		);
	}
}


const WEIGHTS_PER_VERTEX: usize = 2;

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

