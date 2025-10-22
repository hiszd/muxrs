use std::io::Read;

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

/// Returns a `ConfigSchema` from the `muxrs.json` file at the root of the Git repo
#[allow(dead_code)]
pub fn get_config(path: Option<String>, git: bool) -> Result<schema::ConfigSchema, ConfigError> {
  // TODO: Include an option to disable config fallback and fail if not found
  // TODO: Include an option for checking a specified config file
  let path = match git {
    true => {
      let repopath = match git2::Repository::discover(path.clone().unwrap_or(".".to_string())) {
        Ok(r) => r.workdir().unwrap().to_string_lossy().to_string(),
        Err(e) => match e.message().to_string() {
          x if x.contains("could not find repository") => {
            let pth = match path {
              Some(p) => p,
              None => match std::env::home_dir() {
                Some(d) => d.to_string_lossy().to_string(),
                None => return Err(ConfigError::HomeDirNotFound),
              },
            };
            if pth.ends_with("/") {
              pth + ".config/muxrs/muxrs.json"
            } else {
              pth + "/.config/muxrs/muxrs.json"
            }
          }
          _ => return Err(ConfigError::UnknownError(e.to_string())),
        },
      };
      // check to see if the end of the path is a slash, if not add it before muxrs.json
      if repopath.ends_with("/") {
        repopath + "muxrs.json"
      } else {
        repopath + "/muxrs.json"
      }
    }
    false => {
      // NOTE: If there is no path specified and git is false, use the default config from the
      // users home directory
      let pth = match path {
        Some(p) => p,
        None => match std::env::home_dir() {
          Some(d) => d.to_string_lossy().to_string(),
          None => return Err(ConfigError::HomeDirNotFound),
        },
      };
      if pth.ends_with("/") {
        pth + ".config/muxrs/muxrs.json"
      } else {
        pth + "/.config/muxrs/muxrs.json"
      }
    }
  };
  println!("Using config file: {}", path);
  let mut buf = String::new();
  match std::fs::File::open(path) {
    Ok(mut file) => {
      file.read_to_string(&mut buf).unwrap();
    }
    Err(e) => match e.kind() {
      std::io::ErrorKind::NotFound => return Err(ConfigError::ConfigNotFound),
      e => return Err(ConfigError::UnknownError(e.to_string())),
    },
  }
  match serde_json::from_str::<schema::ConfigSchema>(&buf) {
    Ok(json) => Ok(postprocess::extrapolate(json)),
    Err(e) => Err(ConfigError::InvalidConfig(e.to_string())),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn bad_config_location_with_git() {
    match get_config(Some("/etc/".to_string()), true) {
      Err(e) => match e {
        ConfigError::RepoNotFound(_) => assert!(true),
        _ => assert!(false),
      },
      Ok(_) => assert!(false),
    }
  }
  #[test]
  pub fn bad_config_location_without_git() {
    match get_config(Some("/etc/".to_string()), false) {
      Err(e) => match e {
        ConfigError::ConfigNotFound => assert!(true),
        _ => assert!(false),
      },
      Ok(_) => assert!(false),
    }
  }
}
