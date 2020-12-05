use crate::prelude::*;


pub type NavFaceID = usize;
pub type NavEdgeID = usize;
pub type NavVertexID = usize;


#[derive(Debug)]
pub struct NavMesh {
	pub vertices: Vec<NavVertex>,
	pub edges: Vec<NavHalfEdge>,
	pub faces: Vec<NavFace>,
}

#[derive(Debug)]
pub struct NavVertex {
	pub position: Vec3,
	pub outgoing_edge: NavEdgeID,
	pub outgoing_barrier: Option<NavEdgeID>,
}

#[derive(Debug)]
pub struct NavHalfEdge {
	pub vertex: NavVertexID,
	pub next: NavEdgeID,
	pub prev: NavEdgeID,

	pub twin: Option<NavEdgeID>,

	pub face: NavFaceID,
}

#[derive(Debug)]
pub struct NavFace {
	pub start_edge: NavEdgeID,
	pub plane: Plane,
	pub center: Vec3,
}


impl NavMesh {
	pub fn from_entity(entity: toy::EntityRef<'_>) -> NavMesh {
		let mesh_data = entity.mesh_data()
			.expect("entity passed to NavMesh missing mesh data");

		let transform = entity.transform();
		let vertices = mesh_data.positions.iter()
			.map(|&pos| NavVertex {
				position: transform * pos,
				outgoing_edge: 0,
				outgoing_barrier: None
			})
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

		nav_mesh.rebuild_adjacency_info();

		nav_mesh
	}

	fn rebuild_adjacency_info(&mut self) {
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

		// Write outgoing barriers and edges into vertices
		let mut seen_vertices = std::collections::HashSet::new();
		for edge_idx in 0..self.edges.len() {
			let edge = &self.edges[edge_idx];
			let vertex = &mut self.vertices[edge.vertex];

			if seen_vertices.insert(edge.vertex) {
				vertex.outgoing_edge = edge_idx;
			}

			if edge.twin.is_none() {
				assert!(vertex.outgoing_barrier.is_none(), "Vertex found with multiple outgoing barriers!");
				vertex.outgoing_barrier = Some(edge_idx);
			}
		}
	}

	pub fn iter_edge_loop(&self, start_edge: usize) -> impl Iterator<Item=(usize, &'_ NavHalfEdge)> + '_ {
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

	pub fn iter_edge_loop_vertices(&self, start_edge: usize) -> impl Iterator<Item=(usize, &'_ NavVertex)> + '_ {
		self.iter_edge_loop(start_edge)
			.map(move |(_, edge)| (edge.vertex, &self.vertices[edge.vertex]))
	}

	pub fn edge_vertex_positions(&self, edge_idx: usize) -> (Vec3, Vec3) {
		let edge = &self.edges[edge_idx];
		let edge_next = &self.edges[edge.next];

		let vertex_a = self.vertices[edge.vertex].position;
		let vertex_b = self.vertices[edge_next.vertex].position;

		(vertex_a, vertex_b)
	}

	pub fn projected_edge_vertex_positions(&self, edge_idx: usize) -> (Vec2, Vec2) {
		let (edge_a, edge_b) = self.edge_vertex_positions(edge_idx);
		(edge_a.to_xz(), edge_b.to_xz())
	}

	pub fn projected_edge_normal(&self, edge_idx: usize) -> Vec2 {
		let (edge_a, edge_b) = self.projected_edge_vertex_positions(edge_idx);
		(edge_b-edge_a).normalize().perp()
	}

	/// Gives the distance to `point` in worldspace to 2-plane defined by edge
	/// A positive distance means the point lies 'outside' of the edge's face
	/// A negative distance means the point lies 'inside' the edge's face
	pub fn distance_to_projected_edge(&self, edge_idx: usize, point: Vec2) -> f32 {
		let (vertex_a, vertex_b) = self.projected_edge_vertex_positions(edge_idx);

		// edge loops are CCW, so edge_normal will point _away_ from center
		let edge_normal = (vertex_b - vertex_a).normalize().perp();
		(point - vertex_a).dot(edge_normal)
	}

	pub fn projected_edge_loop_contains(&self, start_edge: usize, point: Vec2) -> bool {
		for (edge_idx, _) in self.iter_edge_loop(start_edge) {
			if self.distance_to_projected_edge(edge_idx, point) > 0.0 {
				return false
			}
		}

		true
	}
}

