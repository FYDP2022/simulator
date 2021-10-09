use cgmath::{Vector3, Vector4};
use rusqlite::Connection;

use std::collections::HashMap;

/// Represents a recognized feature
pub struct Feature {
  pub uuid: String,
  pub color: Vector4<u8>,
  pub position_mean: Vector3<f32>,
  pub position_variance: (Vector3<f32>, Vector3<f32>),
  pub radius: f32,
}

pub struct FeatureDB {
  _connection: Connection,
  _features: HashMap<String, Feature>,
}

impl FeatureDB {
  pub fn new() -> Self {
    let connection = Connection::open("features.db").unwrap();
    Self {
      _connection: connection,
      _features: HashMap::new(),
    }
  }
}
