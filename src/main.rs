#![deny(rust_2018_idioms, future_incompatible)]
#![feature(type_ascription)]
#![feature(clamp)]

pub mod prelude;
pub mod gfx;
pub mod window;
pub mod util;

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
		NavMesh::from_entity(nav_ent)
	};

	println!("nav mesh {:#?}", nav_mesh);


	let mut running = true;

	let mut go_forward = false;
	let mut go_backward = false;
	let mut go_left = false;
	let mut go_right = false;
	let mut go_fast = false;

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

							Some(VirtualKeyCode::Escape) => {
								running = false;
							}

							Some(VirtualKeyCode::W) => { go_forward = down; }
							Some(VirtualKeyCode::S) => { go_backward = down; }
							Some(VirtualKeyCode::A) => { go_left = down; }
							Some(VirtualKeyCode::D) => { go_right = down; }
							Some(VirtualKeyCode::LShift) => { go_fast = down; }

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

		let fwd = camera.orientation().forward();
		let right = camera.orientation().right();
		let speed = if go_fast { 4.0 } else { 1.0 } / 60.0;

		let mut camera_delta = Vec3::zero();
		if go_forward { camera_delta += fwd; }
		if go_backward { camera_delta -= fwd; }
		if go_left { camera_delta -= right; }
		if go_right { camera_delta += right; }

		camera.set_position(camera.position() + camera_delta * speed);

		gfx.core.use_shader(shader);
		gfx.core.set_uniform_mat4("u_proj_view", &camera.projection_view());

		gfx.core.set_blend_mode(gfx::core::BlendMode::None);
		gfx.core.draw_mesh(static_mesh);

		draw_nav_mesh(&mut gfx.debug, &nav_mesh);
		draw_nav_intersect(&mut gfx.debug, &nav_mesh, &camera);

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


#[derive(Debug)]
struct NavMesh {
	vertices: Vec<NavVertex>,
	half_edges: Vec<NavHalfEdge>,
	faces: Vec<NavFace>,
}

#[derive(Debug)]
struct NavVertex {
	position: Vec3,
	// outgoing_edge: usize,
}

#[derive(Debug)]
struct NavHalfEdge {
	vertex: usize,
	next: usize,
	prev: usize,

	twin: Option<usize>,

	face: usize,
}

#[derive(Debug)]
struct NavFace {
	start_edge: usize,
	plane: Plane,
	center: Vec3,
}


impl NavMesh {
	fn from_entity(entity: toy::EntityRef<'_>) -> NavMesh {
		let mesh_data = entity.mesh_data()
			.expect("entity passed to NavMesh missing mesh data");

		let transform = entity.transform();
		let vertices = mesh_data.positions.iter()
			.map(|&pos| NavVertex { position: transform * pos })
			.collect();

		let mut half_edges = Vec::with_capacity(mesh_data.indices.len());
		let mut faces = Vec::with_capacity(mesh_data.indices.len() / 3);

		for triangle in mesh_data.indices.chunks(3) {
			let start_edge = half_edges.len();
			let face = faces.len();

			let points = [
				mesh_data.positions[triangle[0] as usize],
				mesh_data.positions[triangle[1] as usize],
				mesh_data.positions[triangle[2] as usize],
			];

			let plane = Plane::from_points(points[0], points[1], points[2]);
			let center = points.iter().sum(): Vec3 / 3.0;

			faces.push(NavFace {
				start_edge,
				plane,
				center,
			});

			half_edges.push(NavHalfEdge {
				vertex: triangle[0] as usize,
				next: start_edge+1,
				prev: start_edge+2,
				twin: None,
				face,
			});

			half_edges.push(NavHalfEdge {
				vertex: triangle[1] as usize,
				next: start_edge+2,
				prev: start_edge,
				twin: None,
				face,
			});

			half_edges.push(NavHalfEdge {
				vertex: triangle[2] as usize,
				next: start_edge,
				prev: start_edge+1,
				twin: None,
				face,
			});
		}

		let mut nav_mesh = NavMesh {
			vertices,
			half_edges,
			faces,
		};

		nav_mesh.recalculate_twins();

		nav_mesh
	}

	fn recalculate_twins(&mut self) {
		use std::collections::HashMap;

		let mut vert_pair_to_edge: HashMap<(usize, usize), usize> = HashMap::new();

		for (edge_idx, edge) in self.half_edges.iter().enumerate() {
			let edge_next = &self.half_edges[edge.next];
			let vert_pair = (edge.vertex, edge_next.vertex);

			if let Some(dupli_edge_idx) = vert_pair_to_edge.insert(vert_pair, edge_idx) {
				panic!("Duplicate half edge! {}, {}", dupli_edge_idx, edge_idx);
			}
		}

		for edge_idx in 0..self.half_edges.len() {
			let edge = &self.half_edges[edge_idx];
			let edge_next = &self.half_edges[edge.next];
			let twin_vert_pair = (edge_next.vertex, edge.vertex);

			let edge = &mut self.half_edges[edge_idx];

			if let Some(twin_idx) = vert_pair_to_edge.get(&twin_vert_pair) {
				edge.twin = Some(*twin_idx);
			} else {
				edge.twin = None;
			}
		}
	}

	fn iter_edge_loop(&self, start_edge: usize) -> impl Iterator<Item=(usize, &'_ NavHalfEdge)> + '_ {
		let final_edge_idx = self.half_edges[start_edge].prev;
		let mut edge_idx = start_edge;

		std::iter::from_fn(move || {
			if edge_idx == final_edge_idx { return None }

			let current_edge_idx = edge_idx;
			let edge = &self.half_edges[current_edge_idx];
			edge_idx = edge.next;

			Some((current_edge_idx, edge))
		}).chain(std::iter::once((final_edge_idx, &self.half_edges[final_edge_idx])))
	}

	fn iter_edge_loop_vertices(&self, start_edge: usize) -> impl Iterator<Item=(usize, &'_ NavVertex)> + '_ {
		self.iter_edge_loop(start_edge)
			.map(move |(_, edge)| (edge.vertex, &self.vertices[edge.vertex]))
	}
}



fn draw_nav_mesh(debug: &mut gfx::debug::Debug, nav: &NavMesh) {
	for v in nav.vertices.iter() {
		debug.point(v.position, Color::rgb(1.0, 0.0, 1.0));
	}

	for &NavFace{start_edge, center, plane} in nav.faces.iter() {
		debug.point(center, Color::rgb(1.0, 1.0, 0.5));

		let basis_length = 0.5;
		let basis_u = plane.project(center + Vec3::from_x(basis_length));
		let basis_v = plane.project(center + Vec3::from_z(-basis_length));

		debug.line(center, basis_u, Color::rgb(0.2, 0.5, 0.5));
		debug.line(center, basis_v, Color::rgb(0.2, 0.5, 0.5));

		debug.line(center, center + plane.normal * basis_length, Color::rgb(0.5, 1.0, 1.0));

		for (_, edge) in nav.iter_edge_loop(start_edge) {
			let edge_next = &nav.half_edges[edge.next];
			let vertex_a = &nav.vertices[edge.vertex];
			let vertex_b = &nav.vertices[edge_next.vertex];

			let pos_a = (0.1).ease_linear(vertex_a.position, center);
			let pos_b = (0.1).ease_linear(vertex_b.position, center);

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

fn draw_nav_intersect(debug: &mut gfx::debug::Debug, nav: &NavMesh, camera: &gfx::camera::Camera) {
	let cam_fwd = camera.orientation().forward();
	let cam_pos = camera.position();

	for &NavFace{plane, center, start_edge} in nav.faces.iter() {
		let intersect = match util::intersect_plane(plane, cam_pos, cam_fwd) {
			Some(intersect) => intersect,
			None => continue
		};

		for (_, edge) in nav.iter_edge_loop(start_edge) {
			let edge_next = &nav.half_edges[edge.next];
			let vertex_a = &nav.vertices[edge.vertex];
			let vertex_b = &nav.vertices[edge_next.vertex];

		}

		// debug.line(intersect, intersect + plane.normal * 0.3, Color::rgb(1.0, 0.8, 0.3));
		// debug.line(intersect, center, Color::rgb(0.6, 1.0, 0.4));
	}

}