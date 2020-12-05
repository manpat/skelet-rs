use crate::prelude::*;

use crate::gfx::camera::Camera;
use crate::nav::{self, NavMesh, NavFaceID};
use crate::util;

pub const PLAYER_HEIGHT: f32 = 2.0;
pub const PLAYER_HEIGHT_VEC: Vec3 = Vec3::from_y(PLAYER_HEIGHT);


#[derive(Debug)]
pub struct PlayerController {
	pub go_forward: bool,
	pub go_backward: bool,
	pub go_left: bool,
	pub go_right: bool,
	pub go_fast: bool,

	fly_mode: bool,

	current_nav_face: Option<NavFaceID>,
}

impl PlayerController {
	pub fn new() -> PlayerController {
		PlayerController {
			go_forward: false,
			go_backward: false,
			go_left: false,
			go_right: false,
			go_fast: false,

			fly_mode: false,

			current_nav_face: None,
		}
	}

	pub fn nav_face(&self) -> Option<NavFaceID> { self.current_nav_face }

	pub fn toggle_fly_mode(&mut self) {
		self.fly_mode = !self.fly_mode;

		if self.fly_mode {
			self.current_nav_face = None;
		}
	}

	pub fn update(&mut self, camera: &mut Camera, nav_mesh: &NavMesh) {
		let speed = if self.go_fast { 6.0 } else { 3.0 } / 60.0;

		if self.fly_mode {
			let fwd = camera.orientation().forward();
			let right = camera.orientation().right();

			let mut camera_delta = Vec3::zero();
			if self.go_forward { camera_delta += fwd; }
			if self.go_backward { camera_delta -= fwd; }
			if self.go_left { camera_delta -= right; }
			if self.go_right { camera_delta += right; }

			camera.set_position(camera.position() + camera_delta * speed);

		} else {
			if self.current_nav_face.is_none() {
				let new_nav_face = get_approx_nearest_nav_face(&nav_mesh, camera.position() - PLAYER_HEIGHT_VEC);
				self.current_nav_face = Some(new_nav_face);
			}

			if let Some(face_idx) = self.current_nav_face {
				let yaw = camera.yaw();
				let right = Vec2::from_angle(-yaw);
				let fwd = -right.perp();

				let mut camera_delta = Vec2::zero();
				if self.go_forward { camera_delta += fwd; }
				if self.go_backward { camera_delta -= fwd; }
				if self.go_left { camera_delta -= right; }
				if self.go_right { camera_delta += right; }

				let new_pos_2d = slide_player_along_barriers(
					&nav_mesh,
					face_idx,
					camera.position().to_xz(),
					camera_delta * speed
				);

				let new_face_idx = transition_player_across_edges(&nav_mesh, face_idx, new_pos_2d);

				self.current_nav_face = Some(new_face_idx);

				let face_plane = nav_mesh.faces[new_face_idx].plane;
				let pos_3d = util::intersect_plane(face_plane, new_pos_2d.to_x0z(), Vec3::from_y(1.0))
					.unwrap();

				camera.set_position(pos_3d + PLAYER_HEIGHT_VEC);
			}
		}
	}
}


fn get_approx_nearest_nav_face(nav: &NavMesh, pos: Vec3) -> NavFaceID {
	let (vertex_dist, nearest_vertex) = nav.vertices.iter()
		.map(|v| ((v.position-pos).length(), v))
		.min_by_key(|(dist, _)| dist.ordify())
		.unwrap();

	let (face_dist, nearest_face_id) = nav.faces.iter() .enumerate()
		.map(|(idx, f)| ((f.center-pos).length(), idx))
		.min_by_key(|(dist, _)| dist.ordify())
		.unwrap();

	if face_dist < vertex_dist {
		nearest_face_id
	} else {
		nav.edges[nearest_vertex.outgoing_edge].face
	}
}

fn projected_plane_rejection(wall_start: Vec2, wall_end: Vec2, point: Vec2) -> Vec2 {
	let wall_diff = wall_end - wall_start;
	let wall_normal = wall_diff.normalize().perp();

	let distance = wall_normal.dot(point - wall_start);

	-wall_normal * distance.max(0.0)
}


fn slide_player_along_barriers(
	nav: &NavMesh, current_face_idx: NavFaceID,
	start_pos: Vec2, mut delta: Vec2) -> Vec2
{
	if delta.length() <= 0.0 { return start_pos; }

	let nav::NavFace{start_edge, ..} = nav.faces[current_face_idx];

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
			// Vertex is convex
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


fn transition_player_across_edges(nav: &NavMesh, current_face_idx: NavFaceID, position: Vec2) -> NavFaceID {
	let nav::NavFace{start_edge, ..} = nav.faces[current_face_idx];

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