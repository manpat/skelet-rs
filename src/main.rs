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
	let mut mouse_pos = Vec2::zero();

	let basic_shader = gfx.core.new_shader(
		include_str!("shaders/basic_vert.glsl"),
		include_str!("shaders/frag.glsl"),
		&["a_vertex", "a_color"]
	);

	let project_data = std::fs::read("../assets/3.toy")?;
	let project = toy::load(&project_data)?;

	let cube_mesh = {
		let toy_ent = project.find_entity("Cube").expect("Missing entity");
		let toy_mesh = toy_ent.mesh_data().expect("missing mesh data");
		gfx.anim.register_animated_mesh(&mut gfx.core, &toy_mesh)
	};

	let bob_anim = gfx::animation::AnimationID(cube_mesh, 0);

	struct Instance {
		pos: Vec3,
		rot: Quat,

		anim_time: f32,
	}

	let mut instances = vec![];

	fn calculate_instance_transform(inst: &Instance) -> Mat4 {
		let car_scale = Mat4::scale(Vec3::splat(0.3));
		Mat4::translate(inst.pos)
			* (Quat::new(Vec3::from_y(1.0), PI/2.0) * inst.rot).to_mat4()
			* car_scale
	}


	'main_loop: loop {
		// gfx.ui.clear_click_state();
		// elapsed += 1.0 / 60.0; 

		let events = window.poll_events();

		for event in events {
			match event {
				WindowEvent::CursorMoved {position, ..} => {
					let PhysicalPosition{x, y} = position.to_physical(window.dpi());
					let pos = Vec2::new(x as f32, y as f32);
					mouse_pos = window_to_screen(window.size(), pos);
					// gfx.ui.on_mouse_move(mouse_pos);
				}

				WindowEvent::MouseInput {state: Pressed, button: LeftMouse, ..} => {
					// gfx.ui.on_mouse_click();

					let near_plane_pos = gfx.camera.inverse_projection_view() * mouse_pos.extend(0.0).extend(1.0);
					let near_plane_pos = near_plane_pos.to_vec3() / near_plane_pos.w;
					let pos = util::intersect_ground(near_plane_pos, gfx.camera.forward());
					let rot = Quat::new(Vec3::from_y(1.0), rand::random::<f32>() * PI * 2.0);

					instances.push(Instance {pos, rot, anim_time: 0.0});
				}

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
			let dt = 1.0/60.0;
			inst.anim_time += 12.0 * dt; // animations[inst.anim_idx].fps * dt;
			inst.pos += inst.rot.forward() * 3.0 * dt;
		}

		instances.retain(|inst| inst.pos.length() < 20.0);


		for inst in instances.iter() {
			gfx.anim.add_instance(gfx::animation::AnimatedMeshInstance {
				transform: calculate_instance_transform(inst),
				animation: bob_anim,
				animation_time: inst.anim_time
			})
		}

		gfx.anim.draw(&mut gfx.core, &gfx.camera);

		// gfx.core.use_shader(basic_shader);
		// gfx.core.set_uniform_mat4("u_proj_view", &gfx.camera.projection_view());
		// gfx.core.draw_mesh(marker_mesh);

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

