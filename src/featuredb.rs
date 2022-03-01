use cgmath::{Matrix4, Vector3};
use rusqlite::{params, Connection, Result, Row, Statement};

/// Represents a recognized feature
pub struct Feature {
  pub id: u32,
  pub n: u32,
  pub age: u32,
  pub color: Vector3<u8>,
  pub position_mean: Vector3<f32>,
  pub position_deviation: Vector3<f32>,
  pub orientation_mean: Vector3<f32>,
  pub orientation_deviation: f32,
  pub radius_mean: f32,
  pub radius_deviation: f32,
  pub material: u8,
}

impl Feature {
  pub fn from_row(row: &Row<'_>) -> Result<Self> {
    Ok(Self {
      id: row.get("id")?,
      n: row.get("n")?,
      age: row.get("age")?,
      color: (row.get("color_r")?, row.get("color_g")?, row.get("color_b")?).into(),
      position_mean: (
        row.get("position_mean_x")?,
        row.get("position_mean_y")?,
        row.get("position_mean_z")?,
      )
        .into(),
      position_deviation: (
        row.get("position_deviation_x")?,
        row.get("position_deviation_y")?,
        row.get("position_deviation_z")?,
      )
        .into(),
      orientation_mean: (
        row.get("orientation_mean_x")?,
        row.get("orientation_mean_y")?,
        row.get("orientation_mean_z")?,
      )
        .into(),
      orientation_deviation: row.get("orientation_deviation")?,
      radius_mean: row.get("radius_mean")?,
      radius_deviation: row.get("radius_deviation")?,
      material: row.get("material")?,
    })
  }

  pub fn transform(&self) -> Matrix4<f32> {
    Matrix4::from_translation(self.position_mean)
      * Matrix4::from_nonuniform_scale(
        self.radius_mean, // self.position_deviation.x + 1.0,
        self.radius_mean, // self.position_deviation.y + 1.0,
        self.radius_mean, // self.position_deviation.z + 1.0,
      )
  }
}

pub struct FeatureDB {
  connection: Connection,
}

impl FeatureDB {
  pub fn new() -> Result<Self> {
    let connection = Connection::open("recognition.sqlite")?;

    connection.execute(
      "CREATE TABLE IF NOT EXISTS features (
        id INTEGER PRIMARY KEY,
        n INTEGER,
        age INTEGER,
        color_r INTEGER,
        color_g INTEGER,
        color_b INTEGER,
        position_mean_x REAL,
        position_mean_y REAL,
        position_mean_z REAL,
        position_deviation_x REAL,
        position_deviation_y REAL,
        position_deviation_z REAL,
        orientation_mean_x REAL,
        orientation_mean_y REAL,
        orientation_mean_z REAL,
        orientation_deviation REAL,
        radius_mean REAL,
        radius_deviation REAL,
        material INTEGER
      )",
      [],
    )?;

    Ok(Self { connection })
  }

  pub fn clear(&self) -> Result<usize> {
    self.connection.execute("DELETE FROM features", [])
  }

  pub fn all(&self) -> Result<Statement<'_>> {
    self
      .connection
      .prepare("SELECT * FROM features")
  }

  pub fn insert(&self, features: Vec<Feature>) -> Result<()> {
    for feature in features {
      self.connection.execute(
        "INSERT INTO features (n, age,
          color_r, color_g, color_b,
          position_mean_x, position_mean_y, position_mean_z,
          position_deviation_x, position_deviation_y, position_deviation_z,
          orientation_mean_x, orientation_mean_y, orientation_mean_z,
          orientation_deviation,
          radius_mean,
          radius_deviation,
          material
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)",
        params![
          feature.n,
          feature.age,
          feature.color.x,
          feature.color.y,
          feature.color.z,
          feature.position_mean.x,
          feature.position_mean.y,
          feature.position_mean.z,
          feature.position_deviation.x,
          feature.position_deviation.y,
          feature.position_deviation.z,
          feature.orientation_mean.z,
          feature.orientation_mean.z,
          feature.orientation_mean.z,
          feature.orientation_deviation,
          feature.radius_mean,
          feature.radius_deviation,
          feature.material
        ],
      )?;
    }
    Ok(())
  }
}
