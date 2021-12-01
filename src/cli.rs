use super::featuredb::{Feature, FeatureDB};

use clap::{Arg, App};
use rand_distr::{Distribution, Normal};
use cgmath::Vector3;

fn rand_f32_tuple3(dist: &impl Distribution<f32>) -> (f32, f32, f32) {
  (
    dist.sample(&mut rand::thread_rng()),
    dist.sample(&mut rand::thread_rng()),
    dist.sample(&mut rand::thread_rng())
  )
}

pub struct Cli {
  generate: Option<String>,
}

impl Cli {
  pub fn new() -> Self {
    let matches = App::new("Lawny Simulator")
      .arg(Arg::with_name("generate")
        .short("g")
        .long("generate")
        .takes_value(true)
        .help("Generate database")
      )
      .get_matches();
    Cli {
      generate: matches.value_of("generate").map(|x| x.into()),
    }
  }

  pub fn run(&self) -> Result<bool, String> {
    if let Some(generate) = &self.generate {
      if generate == "random" {
        let database = FeatureDB::new().map_err(|_| "failed to load feature database".to_owned())?;
        let normal = Normal::new(0.0, 3.0).unwrap();
        let features = (0..100).map(|_| {
          Feature {
            id: 0,
            color: (128, 128, 128).into(),
            position_mean: rand_f32_tuple3(&normal).into(),
            position_variance: Vector3::from(rand_f32_tuple3(&normal)).map(|x| x.abs()),
            radius_mean: 1.0,
            radius_variance: 0.1,
          }
        }).collect();
        database.insert(features).unwrap();
        Ok(true)
      } else {
        Err(format!("invalid arg value '{}', expected 'random'", generate).into())
      }
    } else {
      Ok(false)
    }
  }
}
