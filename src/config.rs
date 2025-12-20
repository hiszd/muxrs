pub mod postprocess;
pub mod schema;
pub mod utils;

use utils::{
  append_path,
  exists_file,
  git_path,
  path_string,
  read_file,
  write_file,
};

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

#[derive(Debug, Clone)]
pub enum PathConf {
  GitWPath(String),
  GitWOPath,
  NoGitWPath(String),
  NoGitWOPath,
}

#[derive(Debug, Clone)]
pub enum ConfigConf {
  Config(String),
  NoConfigWPath(String, bool),
  NoConfigWOPath(bool),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArgumentConf {
  path: PathConf,
  config: ConfigConf,
}

impl ArgumentConf {
  pub fn new(args: crate::Args) -> Self {
    let path = match (!args.no_git, args.path.clone(), args.config.clone()) {
      (true, Some(p), _) => PathConf::GitWPath(p),
      (true, None, _) => PathConf::GitWOPath,
      (false, Some(p), _) => PathConf::NoGitWPath(p),
      (false, None, _) => PathConf::NoGitWOPath,
    };
    let config = match (!args.no_git, args.path, args.config) {
      (_, _, Some(c)) => ConfigConf::Config(c),
      (g, Some(p), None) => ConfigConf::NoConfigWPath(p, g),
      (g, None, None) => ConfigConf::NoConfigWOPath(g),
    };
    Self { path, config }
  }
}

#[allow(dead_code)]
pub fn get_config_path(args: crate::Args) -> Result<String, ConfigError> {
  let conf = ArgumentConf::new(args.clone());
  tracing::info!("Using config: {:#?}", conf);
  match conf.config {
    ConfigConf::Config(c) => {
      tracing::info!("Using specified config file: {}", c);
      Ok(c)
    }
    ConfigConf::NoConfigWPath(p, g) => {
      if g {
        match git_path(&p) {
          Ok(g) => {
            tracing::info!("Using git root from specified path");
            Ok(append_path(&g, "muxrs.json"))
          }
          Err(e) => match e.to_string().as_str() {
            x if x.contains("could not find repository at") => Err(ConfigError::RepoNotFound(p)),
            _ => Err(ConfigError::UnknownError(e.to_string())),
          },
        }
      } else {
        tracing::info!("Using config from specified path");
        Ok(append_path(&p, "muxrs.json"))
      }
    }
    ConfigConf::NoConfigWOPath(g) => {
      let p = path_string(".");
      if g {
        match git_path(&p) {
          Ok(g) => {
            tracing::info!("Using git root from current location");
            Ok(append_path(&g, "muxrs.json"))
          }
          Err(e) => match e.to_string().as_str() {
            x if x.contains("could not find repository at") => Err(ConfigError::RepoNotFound(p)),
            _ => Err(ConfigError::UnknownError(e.to_string())),
          },
        }
      } else {
        tracing::info!("Using current location");
        Ok(append_path(&p, "muxrs.json"))
      }
    }
  }
}

/// Returns a `ConfigSchema` from the `muxrs.json` file at the root of the Git repo
#[allow(dead_code)]
pub fn get_config(args: crate::Args) -> Result<schema::ConfigSchema, ConfigError> {
  // combine the home path for the user with the config path
  let home = std::env::var("HOME").unwrap();
  let backup_path = append_path(&home, ".config/muxrs/muxrs.json");
  let path = match get_config_path(args.clone()) {
    Ok(p) => p,
    Err(e) => match e {
      ConfigError::RepoNotFound(x) => {
        tracing::info!("Using path for config file: {}", &x);
        append_path(&x, "muxrs.json")
      }
      _ => return Err(e),
    },
  };
  tracing::info!("Attempting to use config file: {}", path);
  match exists_file(&path) {
    true => {
      println!("Using config file: {}", path);
      Ok(postprocess::extrapolate(
        serde_json::from_str::<schema::ConfigSchema>(&read_file(&path).unwrap()).unwrap(),
        args.clone(),
      ))
    }
    false => {
      if !args.no_fallback {
        tracing::info!(
          "Config file not found, attempting to use backup config file at: {}",
          backup_path
        );
        if exists_file(&backup_path) {
          println!("Using config file: {}", &backup_path);
          return Ok(postprocess::extrapolate(
            serde_json::from_str::<schema::ConfigSchema>(&read_file(&backup_path).unwrap())
              .unwrap(),
            args.clone(),
          ));
        } else {
          tracing::info!(
            "Backup config file not found. Creating a default one at: {}",
            backup_path
          );
          let config = schema::ConfigSchema::default();
          let json = serde_json::to_string_pretty(&config).unwrap();
          write_file(&backup_path, json).unwrap();
          println!("Using config file: {}", &backup_path);
          return Ok(postprocess::extrapolate(config, args.clone()));
        }
      }
      Err(ConfigError::ConfigNotFound)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn bad_config_location_with_git() {
    let args = crate::Args {
      no_git: false,
      path: Some("/etc/".to_string()),
      populate: false,
      config: None,
      debug: false,
      no_fallback: false,
      no_attach: false,
      verify: false,
    };
    match get_config(args) {
      Err(e) => match e {
        ConfigError::RepoNotFound(_) => (),
        _ => panic!(""),
      },
      Ok(_) => panic!(""),
    }
  }
  #[test]
  pub fn bad_config_location_without_git() {
    let args = crate::Args {
      no_git: false,
      path: Some("/etc/".to_string()),
      populate: false,
      config: None,
      debug: false,
      no_fallback: false,
      no_attach: false,
      verify: false,
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
