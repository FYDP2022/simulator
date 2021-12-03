use cgmath::{Matrix4, Vector3};
use rusqlite::{params, Connection, Result, Row, Statement};

/// Represents a recognized feature
pub struct Feature {
  pub id: u32,
  pub color: Vector3<u8>,
  pub position_mean: Vector3<f32>,
  pub position_variance: Vector3<f32>,
  pub radius_mean: f32,
  pub radius_variance: f32,
}

impl Feature {
  pub fn from_row(row: &Row<'_>) -> Result<Self> {
    Ok(Self {
      id: row.get("id")?,
      color: (row.get("color_r")?, row.get("color_g")?, row.get("color_b")?).into(),
      position_mean: (
        row.get("position_mean_x")?,
        row.get("position_mean_y")?,
        row.get("position_mean_z")?,
      )
        .into(),
      position_variance: (
        row.get("position_variance_x")?,
        row.get("position_variance_y")?,
        row.get("position_variance_z")?,
      )
        .into(),
      radius_mean: row.get("radius_mean")?,
      radius_variance: row.get("radius_variance")?,
    })
  }

  pub fn transform(&self) -> Matrix4<f32> {
    Matrix4::from_translation(self.position_mean)
      * Matrix4::from_nonuniform_scale(
        self.position_variance.x,
        self.position_variance.y,
        self.position_variance.z,
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
        frame INTEGER,
        color_r INTEGER,
        color_g INTEGER,
        color_b INTEGER,
        position_mean_x REAL,
        position_mean_y REAL,
        position_mean_z REAL,
        position_variance_x REAL,
        position_variance_y REAL,
        position_variance_z REAL,
        radius_mean REAL,
        radius_variance REAL,
        CONSTRAINT Id_Frame UNIQUE (id, frame)
      )",
      [],
    )?;

    Ok(Self { connection })
  }

  pub fn current_frame_number(&self) -> Result<u32> {
    let mut stmt = self.connection.prepare("SELECT MAX(frame) FROM features")?;
    let result = stmt.query([])?.next()?.map(|row| row.get(0).unwrap_or(0)).unwrap_or(0);
    Ok(result)
  }

  pub fn current_frame(&self) -> Result<Statement<'_>> {
    self
      .connection
      .prepare("SELECT frame, * FROM features WHERE frame = (SELECT MAX(frame) FROM features)")
  }

  pub fn clear(&self) -> Result<usize> {
    self.connection.execute("DELETE FROM features", [])
  }

  pub fn insert(&self, features: Vec<Feature>) -> Result<()> {
    let frame = self.current_frame_number()?;
    for feature in features {
      self.connection.execute(
        "INSERT INTO features (frame,
          color_r, color_g, color_b,
          position_mean_x, position_mean_y, position_mean_z,
          position_variance_x, position_variance_y, position_variance_z,
          radius_mean,
          radius_variance
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        params![
          frame,
          feature.color.x,
          feature.color.y,
          feature.color.z,
          feature.position_mean.x,
          feature.position_mean.y,
          feature.position_mean.z,
          feature.position_variance.x,
          feature.position_variance.y,
          feature.position_variance.z,
          feature.radius_mean,
          feature.radius_variance
        ],
      )?;
    }
    Ok(())
  }
}
