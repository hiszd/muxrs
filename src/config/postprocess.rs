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

pub fn extrapolate(config: ConfigSchema, args: crate::Args) -> ConfigSchema {
  ConfigSchema {
    session: SessionSchema {
      name: process(config.session.name.clone(), args.clone()).unwrap(),
      starting_dir: {
        config
          .session
          .starting_dir
          .clone()
          .map(|s| process(s.clone(), args.clone()).unwrap())
      },
    },
    windows: {
      config
        .windows
        .iter()
        .map(|w| WindowSchema {
          name: process(w.name.clone(), args.clone()).unwrap(),
          starting_dir: {
            w.starting_dir
              .as_ref()
              .map(|s| process(s.to_string(), args.clone()).unwrap())
          },
          set_active: w.set_active,
          panes: w.panes.clone(),
        })
        .collect::<Vec<WindowSchema>>()
    },
  }
}

fn process(s: String, args: crate::Args) -> Result<String, ConfigPostProcessError> {
  println!("processing: {:?}", &s);
  match find(s.clone()) {
    Some(c) => {
      let mut buf = s;
      for i in c {
        match replace(buf, i, args.clone()) {
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
    Ok(re) => re.captures(&s).map(|c| {
      c.iter().enumerate().fold(Vec::new(), |acc, (i, m)| {
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
      })
    }),
    Err(e) => panic!("{}", e.to_string()),
  }
}

fn replace(s: String, c: Capture, args: crate::Args) -> Result<String, ConfigPostProcessError> {
  let repl = match s.as_str() {
    "%selected_directory%" => match get_replacement(s.clone(), args.clone()) {
      Ok(s) => s,
      Err(_) => return get_replacement("%current_directory%".to_string(), args.clone()),
    },
    _ => get_replacement(s.clone(), args.clone())?,
  };
  Ok(s.replace(&c.cap, &repl))
}

fn get_replacement(s: String, args: crate::Args) -> Result<String, ConfigPostProcessError> {
  match s.as_str() {
    "%current_directory%" => match std::env::current_dir() {
      Ok(d) => Ok(d.to_string_lossy().to_string()),
      Err(e) => Err(ConfigPostProcessError::UnknownError(e.to_string())),
    },
    "%current_directory_short%" => {
      let cdir = std::env::current_dir()
        .unwrap()
        .to_string_lossy()
        .to_string();
      let v: Vec<&str> = cdir.split("/").collect();
      Ok(v.last().unwrap().to_string())
    }
    "%selected_directory%" => match args.config {
      Some(s) => Ok(s),
      None => Err(ConfigPostProcessError::InvalidReplacement(s.clone())),
    },
    "%selected_directory_short%" => match args.config {
      Some(s) => {
        let v: Vec<&str> = s.split("/").collect();
        if s.ends_with("/") {
          Ok(v.get(v.len() - 2).unwrap().to_string())
        } else {
          Ok(v.last().unwrap().to_string())
        }
      }
      None => Err(ConfigPostProcessError::InvalidReplacement(s.clone())),
    },
    _ => Err(ConfigPostProcessError::InvalidReplacement(s.clone())),
  }
}
