use regex::Regex;

use super::schema::{
  ConfigSchema,
  SessionSchema,
  WindowSchema,
};

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct Capture {
  pub cap: String,
  pub start: usize,
  pub end: usize,
}

#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum ConfigPostProcessError {
  #[error("Invalid replacement: {0}")]
  InvalidReplacement(String),
  #[error("Unknown Error: {0}")]
  UnknownError(String),
}

pub fn extrapolate(config: ConfigSchema) -> ConfigSchema {
  ConfigSchema {
    session: SessionSchema {
      name: process(config.session.name).unwrap(),
      starting_dir: {
        match config.session.starting_dir {
          Some(s) => Some(process(s).unwrap()),
          None => None,
        }
      },
    },
    windows: {
      config
        .windows
        .iter()
        .map(|w| WindowSchema {
          name: process(w.name.clone()).unwrap(),
          starting_dir: {
            match &w.starting_dir {
              Some(s) => Some(process(s.to_string()).unwrap()),
              None => None,
            }
          },
          set_active: w.set_active,
          panes: w.panes.clone(),
        })
        .collect::<Vec<WindowSchema>>()
    },
  }
}

fn process(s: String) -> Result<String, ConfigPostProcessError> {
  println!("processing: {:?}", &s);
  match find(s.clone()) {
    Some(c) => {
      let mut buf = String::from(s);
      for i in c {
        match replace(buf, i) {
          Ok(r) => buf = r,
          Err(e) => return Err(e),
        }
      }
      println!("processed: {}", buf);
      Ok(buf.to_string())
    }
    None => Ok(s),
  }
}

fn find(s: String) -> Option<Vec<Capture>> {
  // create a new regex that will test to see if there is any characters surrounded by % and
  // capture them for replacement
  println!("finding: {:?}", s);
  match Regex::new(r"(%[^%]+%)") {
    Ok(re) => match re.captures(&s) {
      Some(c) => Some(c.iter().enumerate().fold(Vec::new(), |acc, (i, m)| {
        if i == 0 {
          return acc;
        }
        let mut vec = acc.clone();
        let n = m.unwrap();
        vec.push(Capture {
          cap: n.as_str().to_string(),
          start: n.start(),
          end: n.end(),
        });
        vec
      })),
      None => None,
    },
    Err(e) => panic!("{}", e.to_string()),
  }
}

fn replace(s: String, c: Capture) -> Result<String, ConfigPostProcessError> {
  let repl = match s.as_str() {
    "%current_directory%" => std::env::current_dir()
      .unwrap()
      .to_string_lossy()
      .to_string(),
    "%current_directory_short%" => {
      let cdir = std::env::current_dir()
        .unwrap()
        .to_string_lossy()
        .to_string();
      let v: Vec<&str> = cdir.split("/").collect();
      v.last().unwrap().to_string()
    }
    _ => return Err(ConfigPostProcessError::InvalidReplacement(s.clone())),
  };
  Ok(s.replace(&c.cap, &repl))
}
