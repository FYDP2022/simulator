use super::geometry::Geometry;

use cgmath::{Point3, Vector2, Vector3};

use std::path::Path;

#[allow(dead_code)]
pub enum MeshFile<'a> {
  Raw(&'a [u8]),
  File(&'a Path),
}

pub struct Mesh {
  pub id: String,
  pub vertices: Vec<Point3<f32>>,
  pub indices: Option<Vec<u16>>,
  pub normals: Option<Vec<Vector3<f32>>>,
  pub uv_coords: Option<Vec<Vector2<f32>>>,
}

impl From<Geometry> for Mesh {
  fn from(geometry: Geometry) -> Self {
    Self {
      id: String::new(),
      vertices: geometry.vertices,
      normals: Some(geometry.normals),
      indices: Some(geometry.indices),
      uv_coords: None,
    }
  }
}
