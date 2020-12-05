#![deny(rust_2018_idioms, future_incompatible)]
#![feature(type_ascription)]
#![feature(clamp)]

pub mod prelude;
pub mod gfx;
pub mod window;
pub mod util;
pub mod nav;
pub mod player_controller;

use prelude::*;
// use glutin::dpi::PhysicalPosition;
use gfx::vertex::ColorVertex;

fn main() -> Result<(), Box<dyn Error>> {
	let mut window = window::Window::new().expect("Failed to create window");
	let mut gfx = gfx::Gfx::new();
	// let mut mouse_pos = Vec2::zero();

	window.set_cursor_capture(true);

	let mut camera = gfx::camera::Camera::new(
		gfx::camera::ProjectionMode::Perspective { fov_y: PI/3.0 },
		// gfx::camera::ViewMode::Orbit { distance: 10.0 }
		gfx::camera::ViewMode::FirstPerson
	);

	let shader = gfx.core.new_shader(
		include_str!("shaders/basic_vert.glsl"),
		include_str!("shaders/frag.glsl"),
		&["a_vertex", "a_color"]
	);

	let project_data = std::fs::read("assets/navtest.toy")?;
	let project = toy::load(&project_data)?;

	let static_mesh = gfx.core.new_mesh();

	{
		let mut mb = gfx::mesh_builder::MeshBuilder::new(static_mesh);

		for entity in project.entities() {
			if entity.name == "nav" { continue }
			if entity.name.starts_with('_') { continue }

			let mesh_data = match entity.mesh_data() {
				Some(md) => md,
				None => continue,
			};

			let transform = entity.transform();
			let color = Color::white();
			let verts = mesh_data.positions.iter()
				.map(|&pos| ColorVertex::new(transform * pos, color.into()))
				.collect(): Vec<_>;

			mb.add_geometry(&verts, &mesh_data.indices);
		}

		mb.commit(&mut gfx.core);
	}

	let nav_mesh = {
		let nav_ent = project.find_entity("nav").expect("can't find nav");
		nav::NavMesh::from_entity(nav_ent)
	};

	// println!("nav mesh {:#?}", nav_mesh);


	let mut player_controller = player_controller::PlayerController::new();

	let mut running = true;

	let mut prev_capture_mouse = true;
	let mut capture_mouse = true;

	while running {
		let window_size = window.size();
		let window_focussed = window.focussed();

		window.poll_events(|event| {
			use glutin::event::{Event, WindowEvent, DeviceEvent, VirtualKeyCode, ElementState};

			if let Event::DeviceEvent{event, ..} = event {
				if !window_focussed {
					return
				}

				match event {
					DeviceEvent::MouseMotion{delta} if capture_mouse => {
						let pitch_limit = PI/2.0;

						let (delta_yaw, delta_pitch) = delta;
						let min_dimension = window_size.x.min(window_size.y) as f32;

						let delta_yaw = PI * delta_yaw as f32 / min_dimension;
						let delta_pitch = PI * delta_pitch as f32 / min_dimension;

						camera.set_yaw(camera.yaw() - delta_yaw);
						camera.set_pitch((camera.pitch() - delta_pitch).clamp(-pitch_limit, pitch_limit));
					}

					DeviceEvent::Key(input) => {
						let down = matches!(input.state, ElementState::Pressed);

						match input.virtual_keycode {
							Some(VirtualKeyCode::F2) if down => {
								capture_mouse = !capture_mouse;
							}

							Some(VirtualKeyCode::V) if down => player_controller.toggle_fly_mode(),

							Some(VirtualKeyCode::Escape) => {
								running = false;
							}

							Some(VirtualKeyCode::W) => { player_controller.go_forward = down; }
							Some(VirtualKeyCode::S) => { player_controller.go_backward = down; }
							Some(VirtualKeyCode::A) => { player_controller.go_left = down; }
							Some(VirtualKeyCode::D) => { player_controller.go_right = down; }
							Some(VirtualKeyCode::LShift) => { player_controller.go_fast = down; }

							_ => {}
						}
					}

					_ => {}
				}

			} else if let Event::WindowEvent{event, ..} = event {
				match event {
					WindowEvent::CloseRequested => {
						running = false;
					}

					_ => {}
				}
			}
		});

		if capture_mouse != prev_capture_mouse {
			window.set_cursor_capture(capture_mouse);
			prev_capture_mouse = capture_mouse;
		}

		gfx.core.set_viewport(window_size);
		gfx.core.set_bg_color(Color::grey(0.1));
		gfx.core.clear();

		camera.update(window_size);
		player_controller.update(&mut camera, &nav_mesh);

		gfx.core.use_shader(shader);
		gfx.core.set_uniform_mat4("u_proj_view", &camera.projection_view());

		gfx.core.set_blend_mode(gfx::core::BlendMode::None);
		gfx.core.draw_mesh(static_mesh);

		draw_nav_mesh(&mut gfx.debug, &nav_mesh);
		draw_nav_intersect(&mut gfx.debug, &nav_mesh, &camera, player_controller.nav_face());

		gfx.anim.draw(&mut gfx.core, &camera);
		gfx.anim.clear();

		gfx.debug.draw(&mut gfx.core, &camera);

		window.swap();
	}

	Ok(())
}

// fn window_to_screen(window_size: Vec2i, pos: Vec2) -> Vec2 {
// 	let window_half = window_size.to_vec2() / 2.0;
// 	(pos - window_half) / window_half * Vec2::new(1.0, -1.0)
// }



fn draw_nav_mesh(debug: &mut gfx::debug::Debug, nav: &nav::NavMesh) {
	for v in nav.vertices.iter() {
		debug.point(v.position, Color::rgb(1.0, 0.0, 1.0));
	}

	for &nav::NavFace{start_edge, center, ..} in nav.faces.iter() {
		debug.point(center, Color::rgb(1.0, 1.0, 0.5));

		for (edge_idx, edge) in nav.iter_edge_loop(start_edge) {
			let (pos_a, pos_b) = nav.edge_vertex_positions(edge_idx);

			let pos_a = (0.1).ease_linear(pos_a, center);
			let pos_b = (0.1).ease_linear(pos_b, center);

			let edge_dir = (pos_b - pos_a).normalize();

			let edge_col = if edge.twin.is_some() {
				Color::rgba(0.2, 0.6, 0.3, 0.5)
			} else {
				Color::rgba(0.5, 0.2, 0.2, 0.5)
			};

			debug.line(pos_a, pos_b, edge_col);
			debug.line(pos_b, pos_b - edge_dir * 0.1 + Vec3::from_y(0.1), edge_col);
			debug.line(pos_b, pos_b - edge_dir * 0.1 - Vec3::from_y(0.1), edge_col);
		}
	}
}


fn draw_nav_intersect(debug: &mut gfx::debug::Debug, nav: &nav::NavMesh, camera: &gfx::camera::Camera, player_nav_face: Option<usize>) {
	let cam_pos = camera.position();
	let cam_down = Vec3::from_y(-1.0);

	if let Some(face_idx) = player_nav_face {
		let face = &nav.faces[face_idx];

		let intersect = match util::intersect_plane(face.plane, cam_pos, cam_down) {
			Some(intersect) => intersect,
			None => return
		};

		debug.line(intersect, face.center, Color::rgb(0.6, 1.0, 0.4));

		for (edge_idx, edge) in nav.iter_edge_loop(face.start_edge) {
			if edge.twin.is_none() {
				let (va, vb) = nav.edge_vertex_positions(edge_idx);
				debug.line(va + Vec3::from_y(0.1), vb + Vec3::from_y(0.1), Color::rgb(0.0, 1.0, 0.7));
			}

			let vertex = &nav.vertices[edge.vertex];
			if vertex.outgoing_barrier.is_none() { continue }

			let outgoing_barrier_idx = vertex.outgoing_barrier.unwrap();
			let outgoing_barrier = &nav.edges[outgoing_barrier_idx];


			let mut prev_incoming_edge_idx = outgoing_barrier.prev;
			let mut prev_incoming_edge = &nav.edges[prev_incoming_edge_idx];

			while let Some(twin_idx) = prev_incoming_edge.twin {
				prev_incoming_edge_idx = nav.edges[twin_idx].prev;
				prev_incoming_edge = &nav.edges[prev_incoming_edge_idx];
			}


			let incoming_normal = nav.projected_edge_normal(prev_incoming_edge_idx);
			let outgoing_normal = nav.projected_edge_normal(outgoing_barrier_idx);

			if incoming_normal.perp().dot(outgoing_normal) <= 0.0 {
				let (va, vb) = nav.edge_vertex_positions(prev_incoming_edge_idx);
				debug.line(va, vb, Color::rgb(1.0, 0.3, 0.5));

				let (va, vb) = nav.edge_vertex_positions(outgoing_barrier_idx);
				debug.line(va, vb, Color::rgb(1.0, 0.3, 0.5));
			}
		}
	}
}
