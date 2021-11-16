use cgmath::{Point3, Vector3};

pub struct Geometry {
  pub vertices: Vec<Point3<f32>>,
  pub normals: Vec<Vector3<f32>>,
  pub indices: Vec<u16>,
}

impl Default for Geometry {
  fn default() -> Self {
    Self {
      vertices: Vec::new(),
      normals: Vec::new(),
      indices: Vec::new(),
    }
  }
}

pub fn uv_sphere(n: u32) -> Geometry {
  const RADIUS: f32 = 1.0;
  let mut geometry = Geometry::default();
  for i in 0..(n + 1) {
    for j in 0..n {
      // Spherical coords with r = RADIUS
      let theta = (j as f32 / n as f32) * 2.0 * std::f32::consts::PI;
      let phi = (i as f32 / n as f32) * std::f32::consts::PI;
      let vertex = (
        RADIUS * phi.sin() * theta.cos(),
        RADIUS * phi.cos(),
        RADIUS * phi.sin() * theta.sin(),
      )
        .into();
      geometry.vertices.push(vertex);
      geometry.normals.push(vertex.to_homogeneous().truncate());
      // Push indices
      if i > 0 && j > 0 {
        let idx = [i * n + j, i * n + (j - 1), (i - 1) * n + j, (i - 1) * n + (j - 1)];
        geometry
          .indices
          .extend_from_slice(&[idx[0] as u16, idx[1] as u16, idx[2] as u16]);
        if i > 1 {
          geometry
            .indices
            .extend_from_slice(&[idx[3] as u16, idx[2] as u16, idx[1] as u16]);
        }
      }
    }
    // Push indices for square that connects end of current segment to start
    if i > 0 {
      let idx = [i * n, (i + 1) * n - 1, (i - 1) * n, i * n - 1];
      geometry
        .indices
        .extend_from_slice(&[idx[0] as u16, idx[1] as u16, idx[2] as u16]);
      if i > 1 {
        geometry
          .indices
          .extend_from_slice(&[idx[3] as u16, idx[2] as u16, idx[1] as u16]);
      }
    }
  }
  geometry
}
