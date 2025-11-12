use crate::config::schema::{
  SessionSchema,
  WindowSchema,
};

pub mod command;
pub mod query;

#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum TmuxError {
  #[error("Could not create session: {0}")]
  CouldNotCreateSession(String),
  #[error("Unknown Error: {0}")]
  UnknownError(String),
}

pub struct Session {
  name: String,
  config: SessionSchema,
}

impl Session {
  pub fn new(session: SessionSchema) -> Result<Session, TmuxError> {
    match query::session_exists(session.name.clone()) {
      Ok(b) => match b {
        false => {
          match command::new_session(
            session.name.clone(),
            Some(session.starting_dir.clone().unwrap_or(".".to_string())),
          ) {
            Ok(_) => {}
            Err(e) => {
              return Err(TmuxError::CouldNotCreateSession(e.to_string()));
            }
          }
          Ok(Session {
            name: session.name.clone(),
            config: session,
          })
        }
        true => Err(TmuxError::CouldNotCreateSession("Session already exists".to_string())),
      },
      Err(e) => Err(TmuxError::UnknownError(e.to_string())),
    }
  }

  /* --------------------------------------------------------- */
  /* -----------------------Queries--------------------------- */
  /* --------------------------------------------------------- */

  // // tmux list-sessions
  // #[allow(dead_code)]
  // pub fn list_sessions() -> Result<Option<Vec<String>>, TmuxError> {
  //   match Command::new("tmux").arg("list-sessions").output() {
  //     Ok(output) => {
  //       let strng = String::from_utf8_lossy(&output.stdout).to_string();
  //       println!("output: {}", strng);
  //       if output.status.success() {
  //         return Ok(Some(Vec::new()));
  //       } else {
  //         return Ok(None);
  //       }
  //     }
  //     Err(e) => Err(TmuxError::UnknownError(e.to_string())),
  //   }
  // }

  /* --------------------------------------------------------- */
  /* -----------------------Actions--------------------------- */
  /* --------------------------------------------------------- */

  pub fn rename_window(&self, index: usize, name: &str) -> Result<(), TmuxError> {
    match command::rename_window(index, &self.name, name) {
      Ok(_) => Ok(()),
      Err(e) => Err(TmuxError::UnknownError(e.to_string())),
    }
  }

  pub fn respawn_window(&self, window_name: &str, path: &str) -> Result<(), TmuxError> {
    match command::respawn_window(&self.name, window_name, path) {
      Ok(_) => Ok(()),
      Err(e) => Err(TmuxError::UnknownError(e.to_string())),
    }
  }

  pub fn new_window(&self, config: &WindowSchema) -> Result<(), TmuxError> {
    match command::new_window(&self.config, config) {
      Ok(_) => Ok(()),
      Err(e) => Err(TmuxError::UnknownError(e.to_string())),
    }
  }

  pub fn kill_window(&self, window_id: &str) -> Result<(), TmuxError> {
    match command::kill_window(&self.config.name, window_id) {
      Ok(_) => Ok(()),
      Err(e) => Err(TmuxError::UnknownError(e.to_string())),
    }
  }

  pub fn split_window(
    &self,
    window_id: &str,
    window_sdir: Option<String>,
    vertical: bool,
  ) -> Result<(), TmuxError> {
    match command::split_window(
      &self.name,
      window_id,
      self.config.starting_dir.clone(),
      window_sdir,
      vertical,
    ) {
      Ok(_) => Ok(()),
      Err(e) => Err(TmuxError::UnknownError(e.to_string())),
    }
  }

  pub fn send_keys(
    &self,
    window: &crate::config::schema::WindowSchema,
    keys: &str,
    pane: Option<usize>,
  ) -> Result<(), TmuxError> {
    match command::send_keys(&self.config.name, &window.name, keys, pane) {
      Ok(_) => Ok(()),
      Err(e) => Err(TmuxError::UnknownError(e.to_string())),
    }
  }

  pub fn select_window(&self, window: &WindowSchema) -> Result<(), TmuxError> {
    match command::select_window(&self.config.name, &window.name) {
      Ok(_) => Ok(()),
      Err(e) => Err(TmuxError::UnknownError(e.to_string())),
    }
  }
}
