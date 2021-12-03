use super::featuredb::{Feature, FeatureDB};

use cgmath::Vector3;
use clap::{App, Arg};
use rand_distr::{Distribution, Normal};

fn rand_f32_tuple3(dist: &impl Distribution<f32>) -> (f32, f32, f32) {
  (
    dist.sample(&mut rand::thread_rng()),
    dist.sample(&mut rand::thread_rng()),
    dist.sample(&mut rand::thread_rng()),
  )
}

pub struct Cli {
  generate: Option<String>,
  clear: bool,
}

impl Cli {
  pub fn new() -> Self {
    let matches = App::new("Lawny Simulator")
      .arg(
        Arg::with_name("generate")
          .short("g")
          .long("generate")
          .takes_value(true)
          .help("Generate database"),
      )
      .arg(
        Arg::with_name("clear")
          .short("c")
          .long("clear")
          .takes_value(false)
          .help("Clears database"),
      )
      .get_matches();
    Cli {
      generate: matches.value_of("generate").map(|x| x.into()),
      clear: matches.is_present("clear"),
    }
  }

  pub fn run(&self) -> Result<bool, String> {
    let mut cli_mode = false;
    let database = FeatureDB::new().map_err(|_| "failed to load feature database".to_owned())?;
    if self.clear {
      database
        .clear()
        .map_err(|err| format!("failed to clear database: '{}'", err))?;
      cli_mode = true;
    }
    if let Some(generate) = &self.generate {
      if generate == "random" {
        let mean = Normal::new(0.0, 3.0).unwrap();
        let variance = Normal::new(1.0, 0.1).unwrap();
        let features = (0..100)
          .map(|_| Feature {
            id: 0,
            color: (128, 128, 128).into(),
            position_mean: rand_f32_tuple3(&mean).into(),
            position_variance: Vector3::from(rand_f32_tuple3(&variance)).map(|x| x.abs()),
            radius_mean: 1.0,
            radius_variance: 0.1,
          })
          .collect();
        database.insert(features).unwrap();
        cli_mode = true;
      } else {
        return Err(format!("invalid arg value '{}', expected 'random'", generate));
      }
    }

    Ok(cli_mode)
  }
}
