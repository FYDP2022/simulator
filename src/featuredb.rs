use cgmath::{Vector2, Vector3, Vector4};
use rusqlite::{Connection, Result, Rows};

use std::collections::HashMap;
use std::iter::Iterator;

/// Represents a recognized feature
pub struct Feature {
  uuid: String,
  color: Vector4<u8>,
  position_mean: Vector3<f32>,
  position_variance: (Vector3<f32>, Vector3<f32>),
  radius: f32,
}

pub struct FeatureDB {
  connection: Connection,
  features: HashMap<String, Feature>,
}

impl FeatureDB {
  pub fn new() -> Self {
    let connection = Connection::open("features.db").unwrap();
    Self {
      connection,
      features: HashMap::new(),
    }
  }
}
