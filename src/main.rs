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

	let mut fly_cam = false;
	let mut player_nav_face = None;

	const PLAYER_HEIGHT: f32 = 2.0;

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

							Some(VirtualKeyCode::V) if down => {
								fly_cam = !fly_cam;
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

		let speed = if go_fast { 6.0 } else { 3.0 } / 60.0;

		if fly_cam {
			let fwd = camera.orientation().forward();
			let right = camera.orientation().right();

			let mut camera_delta = Vec3::zero();
			if go_forward { camera_delta += fwd; }
			if go_backward { camera_delta -= fwd; }
			if go_left { camera_delta -= right; }
			if go_right { camera_delta += right; }

			camera.set_position(camera.position() + camera_delta * speed);
			player_nav_face = None;

		} else {
			if !player_nav_face.is_some() {
				player_nav_face = get_nearest_projected_nav_face(&nav_mesh, camera.position());

				if let Some(face_idx) = player_nav_face {
					camera.set_position(nav_mesh.faces[face_idx].center + Vec3::from_y(PLAYER_HEIGHT));
				}
			}

			if let Some(face_idx) = player_nav_face {
				let yaw = camera.yaw();
				let right = Vec2::from_angle(-yaw);
				let fwd = -right.perp();

				let mut camera_delta = Vec2::zero();
				if go_forward { camera_delta += fwd; }
				if go_backward { camera_delta -= fwd; }
				if go_left { camera_delta -= right; }
				if go_right { camera_delta += right; }

				let new_pos_2d = slide_player_along_barriers(
					&nav_mesh,
					face_idx,
					camera.position().to_xz(),
					camera_delta * speed
				);

				let new_face_idx = transition_player_across_edges(&nav_mesh, face_idx, new_pos_2d);

				player_nav_face = Some(new_face_idx);

				let face_plane = nav_mesh.faces[new_face_idx].plane;
				let pos_3d = util::intersect_plane(face_plane, new_pos_2d.to_x0z(), Vec3::from_y(1.0))
					.unwrap();

				camera.set_position(pos_3d + Vec3::from_y(PLAYER_HEIGHT));
			}
		}

		gfx.core.use_shader(shader);
		gfx.core.set_uniform_mat4("u_proj_view", &camera.projection_view());

		gfx.core.set_blend_mode(gfx::core::BlendMode::None);
		gfx.core.draw_mesh(static_mesh);

		draw_nav_mesh(&mut gfx.debug, &nav_mesh);
		draw_nav_intersect(&mut gfx.debug, &nav_mesh, &camera, player_nav_face);

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
	edges: Vec<NavHalfEdge>,
	faces: Vec<NavFace>,
}

#[derive(Debug)]
struct NavVertex {
	position: Vec3,
	// outgoing_edge: usize,
	outgoing_barrier: Option<usize>,
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
			.map(|&pos| NavVertex { position: transform * pos, outgoing_barrier: None })
			.collect();

		let mut edges = Vec::with_capacity(mesh_data.indices.len());
		let mut faces = Vec::with_capacity(mesh_data.indices.len() / 3);

		for triangle in mesh_data.indices.chunks(3) {
			let start_edge = edges.len();
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

			edges.push(NavHalfEdge {
				vertex: triangle[0] as usize,
				next: start_edge+1,
				prev: start_edge+2,
				twin: None,
				face,
			});

			edges.push(NavHalfEdge {
				vertex: triangle[1] as usize,
				next: start_edge+2,
				prev: start_edge,
				twin: None,
				face,
			});

			edges.push(NavHalfEdge {
				vertex: triangle[2] as usize,
				next: start_edge,
				prev: start_edge+1,
				twin: None,
				face,
			});
		}

		let mut nav_mesh = NavMesh {
			vertices,
			edges,
			faces,
		};

		nav_mesh.recalculate_twins();

		nav_mesh
	}

	fn recalculate_twins(&mut self) {
		use std::collections::HashMap;

		let mut vert_pair_to_edge: HashMap<(usize, usize), usize> = HashMap::new();

		// Build twins
		for (edge_idx, edge) in self.edges.iter().enumerate() {
			let edge_next = &self.edges[edge.next];
			let vert_pair = (edge.vertex, edge_next.vertex);

			if let Some(dupli_edge_idx) = vert_pair_to_edge.insert(vert_pair, edge_idx) {
				panic!("Duplicate half edge! {}, {}", dupli_edge_idx, edge_idx);
			}
		}

		for edge_idx in 0..self.edges.len() {
			let edge = &self.edges[edge_idx];
			let edge_next = &self.edges[edge.next];
			let twin_vert_pair = (edge_next.vertex, edge.vertex);

			let edge = &mut self.edges[edge_idx];

			if let Some(twin_idx) = vert_pair_to_edge.get(&twin_vert_pair) {
				edge.twin = Some(*twin_idx);
			} else {
				edge.twin = None;
			}
		}

		// Write barriers into vertices
		for edge_idx in 0..self.edges.len() {
			let edge = &self.edges[edge_idx];
			if edge.twin.is_some() { continue }

			let vertex = &mut self.vertices[edge.vertex];
			assert!(vertex.outgoing_barrier.is_none(), "Vertex found with multiple outgoing barriers!");
			vertex.outgoing_barrier = Some(edge_idx);
		}
	}

	fn iter_edge_loop(&self, start_edge: usize) -> impl Iterator<Item=(usize, &'_ NavHalfEdge)> + '_ {
		let final_edge_idx = self.edges[start_edge].prev;
		let mut edge_idx = start_edge;

		std::iter::from_fn(move || {
			if edge_idx == final_edge_idx { return None }

			let current_edge_idx = edge_idx;
			let edge = &self.edges[current_edge_idx];
			edge_idx = edge.next;

			Some((current_edge_idx, edge))
		}).chain(std::iter::once((final_edge_idx, &self.edges[final_edge_idx])))
	}

	fn iter_edge_loop_vertices(&self, start_edge: usize) -> impl Iterator<Item=(usize, &'_ NavVertex)> + '_ {
		self.iter_edge_loop(start_edge)
			.map(move |(_, edge)| (edge.vertex, &self.vertices[edge.vertex]))
	}

	fn edge_vertex_positions(&self, edge_idx: usize) -> (Vec3, Vec3) {
		let edge = &self.edges[edge_idx];
		let edge_next = &self.edges[edge.next];

		let vertex_a = self.vertices[edge.vertex].position;
		let vertex_b = self.vertices[edge_next.vertex].position;

		(vertex_a, vertex_b)
	}

	fn projected_edge_vertex_positions(&self, edge_idx: usize) -> (Vec2, Vec2) {
		let (edge_a, edge_b) = self.edge_vertex_positions(edge_idx);
		(edge_a.to_xz(), edge_b.to_xz())
	}

	fn projected_edge_normal(&self, edge_idx: usize) -> Vec2 {
		let (edge_a, edge_b) = self.projected_edge_vertex_positions(edge_idx);
		(edge_b-edge_a).normalize().perp()
	}

	/// Gives the distance to `point` in worldspace to 2-plane defined by edge
	/// A positive distance means the point lies 'outside' of the edge's face
	/// A negative distance means the point lies 'inside' the edge's face
	fn distance_to_projected_edge(&self, edge_idx: usize, point: Vec2) -> f32 {
		let (vertex_a, vertex_b) = self.projected_edge_vertex_positions(edge_idx);

		// edge loops are CCW, so edge_normal will point _away_ from center
		let edge_normal = (vertex_b - vertex_a).normalize().perp();
		(point - vertex_a).dot(edge_normal)
	}

	fn projected_edge_loop_contains(&self, start_edge: usize, point: Vec2) -> bool {
		for (edge_idx, _) in self.iter_edge_loop(start_edge) {
			if self.distance_to_projected_edge(edge_idx, point) > 0.0 {
				return false
			}
		}

		true
	}
}



fn draw_nav_mesh(debug: &mut gfx::debug::Debug, nav: &NavMesh) {
	for v in nav.vertices.iter() {
		debug.point(v.position, Color::rgb(1.0, 0.0, 1.0));
	}

	for &NavFace{start_edge, center, plane} in nav.faces.iter() {
		debug.point(center, Color::rgb(1.0, 1.0, 0.5));

		// let basis_length = 0.5;
		// let basis_u = plane.project(center + Vec3::from_x(basis_length));
		// let basis_v = plane.project(center + Vec3::from_z(-basis_length));

		// debug.line(center, basis_u, Color::rgb(0.2, 0.5, 0.5));
		// debug.line(center, basis_v, Color::rgb(0.2, 0.5, 0.5));

		// debug.line(center, center + plane.normal * basis_length, Color::rgb(0.5, 1.0, 1.0));

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


fn draw_nav_intersect(debug: &mut gfx::debug::Debug, nav: &NavMesh, camera: &gfx::camera::Camera, player_nav_face: Option<usize>) {
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


fn get_nearest_projected_nav_face(nav: &NavMesh, pos: Vec3) -> Option<usize> {
	for (face_idx, &NavFace{start_edge, ..}) in nav.faces.iter().enumerate() {
		if nav.projected_edge_loop_contains(start_edge, pos.to_xz()) {
			return Some(face_idx)
		}
	}

	None
}


fn projected_plane_rejection(wall_start: Vec2, wall_end: Vec2, point: Vec2) -> Vec2 {
	let wall_diff = wall_end - wall_start;
	let wall_normal = wall_diff.normalize().perp();

	let distance = wall_normal.dot(point - wall_start);

	-wall_normal * distance.max(0.0)
}


fn slide_player_along_barriers(
	nav: &NavMesh, current_face_idx: usize,
	start_pos: Vec2, mut delta: Vec2) -> Vec2
{
	if delta.length() <= 0.0 { return start_pos; }

	let NavFace{start_edge, ..} = nav.faces[current_face_idx];

	for (edge_idx, edge) in nav.iter_edge_loop(start_edge) {
		// If this edge is a barrier, resolve collision
		if edge.twin.is_none() {
			let (va, vb) = nav.projected_edge_vertex_positions(edge_idx);
			delta += projected_plane_rejection(va, vb, start_pos + delta);
		}
		
		// Find any barriers connected to this edge's vertex
		let vertex = &nav.vertices[edge.vertex];
		if vertex.outgoing_barrier.is_none() {
			continue
		}

		let outgoing_barrier_idx = vertex.outgoing_barrier.unwrap();
		let outgoing_barrier = &nav.edges[outgoing_barrier_idx];

		let mut prev_incoming_edge_idx = outgoing_barrier.prev;
		let mut prev_incoming_edge = &nav.edges[prev_incoming_edge_idx];

		while let Some(twin_idx) = prev_incoming_edge.twin {
			prev_incoming_edge_idx = nav.edges[twin_idx].prev;
			prev_incoming_edge = &nav.edges[prev_incoming_edge_idx];
		}

		let incoming_barrier_idx = prev_incoming_edge_idx;


		// Test concavity - if vertex is concave then collide with barriers as planes
		let incoming_normal = nav.projected_edge_normal(incoming_barrier_idx);
		let outgoing_normal = nav.projected_edge_normal(outgoing_barrier_idx);

		if incoming_normal.perp().dot(outgoing_normal) <= 0.0 {
			let (va, vb) = nav.projected_edge_vertex_positions(incoming_barrier_idx);
			delta += projected_plane_rejection(va, vb, start_pos + delta);

			let (va, vb) = nav.projected_edge_vertex_positions(outgoing_barrier_idx);
			delta += projected_plane_rejection(va, vb, start_pos + delta);
		} else {
			// Vertex is concave
			let end_pos = start_pos + delta;
			let vertex_delta = end_pos - vertex.position.to_xz();

			let incoming_dist = incoming_normal.dot(vertex_delta);
			let outgoing_dist = outgoing_normal.dot(vertex_delta);

			// If endpoint is outside *both* barriers, then adjust by the least required distance
			// Just enough to allow this test to pass again
			if incoming_dist >= 0.0 && outgoing_dist >= 0.0 {
				if incoming_dist < outgoing_dist {
					delta -= incoming_normal * incoming_dist;
				} else {
					delta -= outgoing_normal * outgoing_dist;
				}
			}
		}
	}

	start_pos + delta
}


fn transition_player_across_edges(nav: &NavMesh, current_face_idx: usize, position: Vec2) -> usize {
	let NavFace{start_edge, ..} = nav.faces[current_face_idx];

	// Find nearest edge to transition across
	let transition_edge = nav.iter_edge_loop(start_edge)
		.filter(|(_, edge)| edge.twin.is_some())
		.map(move |(edge_idx, edge)| {
			let dist = nav.distance_to_projected_edge(edge_idx, position);
			(edge_idx, edge, dist)
		})
		.filter(|&(_, _, dist)| dist >= 0.0)
		.min_by_key(|&(_, _, dist)| dist.ordify());

	if let Some((_, edge, _)) = transition_edge {
		nav.edges[edge.twin.unwrap()].face
	} else {
		current_face_idx
	}
}