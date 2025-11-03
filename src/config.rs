use std::io::{
  Read,
  Write,
};

pub mod postprocess;
pub mod schema;

#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
  #[error("Invalid config: {0}")]
  InvalidConfig(String),
  #[error("Config file not found")]
  ConfigNotFound,
  #[error("Repository not found: {0}")]
  RepoNotFound(String),
  #[error("Users home directory not found")]
  HomeDirNotFound,
  #[error("Unknown Error: {0}")]
  UnknownError(String),
}

#[allow(dead_code)]
pub fn get_config_path(args: crate::Args) -> Result<String, ConfigError> {
  let git = args.git;
  let configpath = args.config.clone().unwrap_or(".".to_string());
  let homepath = std::env::home_dir().unwrap().to_string_lossy().to_string();
  let mut gitpath = None;
  if git && let Ok(r) = git2::Repository::discover(configpath.clone()) {
    gitpath = Some(r.workdir().unwrap().to_string_lossy().to_string())
  }

  if gitpath.is_some() {
    let mut g = gitpath.unwrap();
    if g.ends_with("/") {
      g += "muxrs.json"
    } else {
      g += "/muxrs.json"
    }
    println!("Checking to see if config file exists: {}", g);
    match std::fs::exists(&g) {
      Ok(true) => {
        println!("Config file found at: {}", g);
        return Ok(g);
      }
      _ => {
        println!("Config file not found at: {}", g);
      }
    }
  }

  let mut pth = if configpath.ends_with("/") {
    configpath + "muxrs.json"
  } else {
    configpath + "/muxrs.json"
  };
  println!("Checking to see if config file exists: {}", pth);
  match std::fs::exists(&pth) {
    Ok(true) => {
      println!("Config file found at: {}", pth);
      return Ok(pth);
    }
    _ => {
      println!("Config file not found at: {}", pth);
    }
  }

  pth = if homepath.ends_with("/") {
    homepath + ".config/muxrs/muxrs.json"
  } else {
    homepath + "/.config/muxrs/muxrs.json"
  };
  println!("Recommending the file to config is: {}", pth);
  Ok(pth)
}

/// Returns a `ConfigSchema` from the `muxrs.json` file at the root of the Git repo
#[allow(dead_code)]
pub fn get_config(args: crate::Args) -> Result<schema::ConfigSchema, ConfigError> {
  let path = get_config_path(args.clone())?;
  println!("Attempting to use config file: {}", path);
  let mut buf = String::new();
  match std::fs::File::open(path) {
    Ok(mut file) => {
      file.read_to_string(&mut buf).unwrap();
    }
    Err(e) => match e.kind() {
      std::io::ErrorKind::NotFound => {
        // store the value of the env variable XDG_CONFIG_HOME
        let confighome = match std::env::var("XDG_CONFIG_HOME") {
          Ok(v) => v,
          Err(e) => match e {
            std::env::VarError::NotPresent => {
              println!("XDG_CONFIG_HOME not set, using home directory instead");
              match std::env::home_dir() {
                Some(d) => {
                  let b = d.to_string_lossy().to_string();
                  if b.ends_with("/") {
                    b + ".config"
                  } else {
                    b + "/.config"
                  }
                }
                None => return Err(ConfigError::HomeDirNotFound),
              }
            }
            _ => return Err(ConfigError::UnknownError(e.to_string())),
          },
        };
        let p = if confighome.ends_with("/") {
          confighome + "muxrs/muxrs.json"
        } else {
          confighome + "/muxrs/muxrs.json"
        };
        println!("Attempting to use default config file");
        match std::fs::File::open(p.clone()) {
          Ok(mut file) => {
            file.read_to_string(&mut buf).unwrap();
          }
          Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => match std::fs::File::create(p) {
              Ok(mut f) => {
                let defaultconfig = schema::ConfigSchema::default();
                match f.write_all(
                  serde_json::to_string_pretty(&defaultconfig)
                    .unwrap()
                    .as_bytes(),
                ) {
                  Ok(_) => return Ok(postprocess::extrapolate(defaultconfig, args.clone())),
                  Err(e) => return Err(ConfigError::UnknownError(e.to_string())),
                }
              }
              Err(e) => return Err(ConfigError::UnknownError(e.to_string())),
            },
            _ => return Err(ConfigError::UnknownError(e.to_string())),
          },
        }
      }
      e => return Err(ConfigError::UnknownError(e.to_string())),
    },
  }
  match serde_json::from_str::<schema::ConfigSchema>(&buf) {
    Ok(json) => Ok(postprocess::extrapolate(json, args.clone())),
    Err(e) => Err(ConfigError::InvalidConfig(e.to_string())),
  }
}
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn bad_config_location_with_git() {
    let args = crate::Args {
      git: true,
      config: Some("/etc/".to_string()),
      debug: false,
      no_fallback: false,
    };
    match get_config(args) {
      Err(e) => match e {
        ConfigError::RepoNotFound(_) => panic!(""),
        _ => panic!(""),
      },
      Ok(_) => panic!(""),
    }
  }
  #[test]
  pub fn bad_config_location_without_git() {
    let args = crate::Args {
      git: true,
      config: Some("/etc/".to_string()),
      debug: false,
      no_fallback: false,
    };
    match get_config(args) {
      Err(e) => match e {
        ConfigError::ConfigNotFound => panic!(""),
        _ => panic!(""),
      },
      Ok(_) => panic!(""),
    }
  }
}
