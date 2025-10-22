use std::process::Command;

use regex::Regex;

use crate::config::schema::SessionSchema;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TmuxSession {
  name: String,
  windows: Vec<TmuxWindow>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TmuxWindow {
  name: String,
  active: bool,
  last: bool,
  panes: usize,
}

#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum TmuxQueryError {
  #[error("Unknown Error: {0}")]
  UnknownError(String),
}

pub fn get_windows(session: &SessionSchema) -> Result<Vec<TmuxWindow>, TmuxQueryError> {
  match Command::new("tmux")
    .arg("list-windows")
    .arg("-t")
    .arg(session.name.clone())
    .output()
  {
    Ok(output) => {
      let strng = String::from_utf8_lossy(&output.stdout).to_string();
      println!("list-windows");
      // go over each line of the output
      let lines = strng.lines().collect::<Vec<&str>>();
      let mut windows = Vec::new();
      for line in lines {
        // get the name of the window by starting at character 2
        // the lines could look like any of the following lines:
        // 0: nvim (2 panes) [254x66] [layout 0bdc,254x66,0,0{127x66,0,0,69,126x66,128,0,70}] @64
        // 1: terminal- (1 panes) [254x66] [layout 6523,254x66,0,0,71] @65
        // 2: fish* (1 panes) [254x66] [layout 6526,254x66,0,0,74] @68 (active)
        let re =
          Regex::new(r"(?P<num>\d): (?P<name>\w+)(?P<indicator>\W?)\s\((?P<panes>\d+) panes\)")
            .unwrap();
        let mat = match re.captures(line) {
          Some(mat) => mat,
          None => return Err(TmuxQueryError::UnknownError("Could not parse line".to_string())),
        };
        let indicator = match mat.name("indicator") {
          Some(indicator) => indicator.as_str(),
          None => " ",
        };
        let wind = TmuxWindow {
          name: mat.name("name").unwrap().as_str().to_string(),
          panes: mat.name("panes").unwrap().as_str().parse().unwrap(),
          active: indicator == "*",
          last: indicator == "-",
        };
        windows.push(wind);
      }
      Ok(windows)
    }
    Err(e) => {
      println!("Error: {}", e.to_string());
      Err(TmuxQueryError::UnknownError(e.to_string()))
    }
  }
}

// pub fn get_sessions() -> Result<Vec, TmuxEncodeError> {
// }

// tmux has-session -t {name}
/// Returns true if the session exists
#[allow(dead_code)]
pub fn session_exists(name: String) -> Result<bool, TmuxQueryError> {
  match Command::new("tmux")
    .arg("has-session")
    .arg("-t")
    .arg(name.clone())
    .output()
  {
    Ok(output) => {
      let strng = String::from_utf8_lossy(&output.stdout).to_string();
      println!("output: {}, status: {}", strng, output.status);
      if output.status.code().unwrap() == 0 {
        return Ok(true);
      } else {
        return Ok(false);
      }
    }
    Err(e) => Err(TmuxQueryError::UnknownError(e.to_string())),
  }
}
