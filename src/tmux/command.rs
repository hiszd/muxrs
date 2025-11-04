use std::process::Command;

use crate::config::schema::{SessionSchema, WindowSchema};

#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum TmuxCommandError {
  #[error("Could not create session: {0}")]
  CouldNotCreateSession(String),
  #[error("Unknown Error: {0}")]
  UnknownError(String),
}

// tmux new-session -d -s {session_name}
#[allow(dead_code)]
pub fn new_session(name: String, start_dir: Option<String>) -> Result<(), TmuxCommandError> {
  let mut cmd = Command::new("tmux");
  cmd.arg("new-session").arg("-d").arg("-s").arg(name);
  if let Some(sd) = start_dir {
    cmd.arg("-c").arg(sd);
  }
  match cmd.output() {
    Ok(output) => {
      let strng = String::from_utf8_lossy(&output.stdout).to_string();
      println!("output: {}", strng);
      Ok(())
    }
    Err(e) => Err(TmuxCommandError::CouldNotCreateSession(e.to_string())),
  }
}

// tmux rename-window -t {session}:0 {window_name}
#[allow(dead_code)]
pub fn rename_window(index: usize, name: &str, new_name: &str) -> Result<(), TmuxCommandError> {
  match Command::new("tmux")
    .arg("rename-window")
    .arg("-t")
    .arg(format!("{}:{}", name, index))
    .arg(new_name)
    .output()
  {
    Ok(output) => {
      let strng = String::from_utf8_lossy(&output.stdout).to_string();
      println!("output: {}", strng);
      Ok(())
    }
    Err(e) => Err(TmuxCommandError::UnknownError(e.to_string())),
  }
}

// tmux new-window -t {session}:{index} -n {window_name}
#[allow(dead_code)]
pub fn new_window(
  session: &crate::config::schema::SessionSchema,
  window: &crate::config::schema::WindowSchema,
) -> Result<(), TmuxCommandError> {
  let path = match window.starting_dir.clone() {
    Some(p) => p,
    None => session.starting_dir.clone().unwrap_or(".".to_string()),
  };
  match Command::new("tmux")
    .arg("new-window")
    .arg("-t")
    .arg(session.name.clone())
    .arg("-n")
    .arg(window.name.clone())
    .arg("-c")
    .arg(path)
    .output()
  {
    Ok(output) => {
      let strng = String::from_utf8_lossy(&output.stdout).to_string();
      println!("output: {}", strng);
      Ok(())
    }
    Err(e) => Err(TmuxCommandError::UnknownError(e.to_string())),
  }
}

// tmux split-window -t {session}:{window}
#[allow(dead_code)]
pub fn split_window(name: &str, window: &str, vertical: bool) -> Result<(), TmuxCommandError> {
  match Command::new("tmux")
    .arg("split-window")
    .arg("-t")
    .arg(format!("{}:{}", name, window))
    .arg(if vertical { "-v" } else { "-h" })
    .output()
  {
    Ok(output) => {
      let strng = String::from_utf8_lossy(&output.stdout).to_string();
      println!("output: {}", strng);
      Ok(())
    }
    Err(e) => Err(TmuxCommandError::UnknownError(e.to_string())),
  }
}

pub fn send_keys(
  session: &crate::config::schema::SessionSchema,
  window: &crate::config::schema::WindowSchema,
  keys: &str,
  pane: Option<usize>,
) -> Result<(), TmuxCommandError> {
  let id = match pane {
    Some(p) => format!("{}:{}.{}", session.name.clone(), window.name, p),
    None => format!("{}:{}", session.name.clone(), window.name),
  };
  match Command::new("tmux")
    .arg("send-keys")
    .arg("-t")
    .arg(id)
    .arg(keys)
    .arg("C-m")
    .output()
  {
    Ok(output) => {
      let strng = String::from_utf8_lossy(&output.stdout).to_string();
      println!("output: {}", strng);
      Ok(())
    }
    Err(e) => Err(TmuxCommandError::UnknownError(e.to_string())),
  }
}

// tmux focus-window -t {sessionname}:{windowname}
#[allow(dead_code)]
pub fn select_window(
  session: &SessionSchema,
  window: &WindowSchema,
) -> Result<(), TmuxCommandError> {
  match Command::new("tmux")
    .arg("select-window")
    .arg("-t")
    .arg(format!("{}:{}", &session.name, &window.name))
    .output()
  {
    Ok(output) => {
      let strng = String::from_utf8_lossy(&output.stdout).to_string();
      println!("output: {}", strng);
      Ok(())
    }
    Err(e) => Err(TmuxCommandError::UnknownError(e.to_string())),
  }
}

// tmux focus-window -t {sessionname}:{windowname}
#[allow(dead_code)]
pub fn attach(session: &SessionSchema) -> Result<(), TmuxCommandError> {
  match Command::new("tmux")
    .arg("attach")
    .arg("-t")
    .arg(&session.name)
    .output()
  {
    Ok(output) => {
      let strng = String::from_utf8_lossy(&output.stdout).to_string();
      println!("output: {}", strng);
      Ok(())
    }
    Err(e) => Err(TmuxCommandError::UnknownError(e.to_string())),
  }
}
