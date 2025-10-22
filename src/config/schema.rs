use serde::{
  Deserialize,
  Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigSchema {
  pub session: SessionSchema,
  #[serde(default)]
  pub windows: Vec<WindowSchema>,
}

impl ConfigSchema {
  #[allow(dead_code)]
  pub fn new(session: SessionSchema, windows: Vec<WindowSchema>) -> ConfigSchema {
    ConfigSchema { session, windows }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionSchema {
  pub name: String,
  pub starting_dir: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowSchema {
  pub name: String,
  pub starting_dir: Option<String>,
  pub set_active: Option<bool>,
  #[serde(default)]
  pub panes: Vec<PaneSchema>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaneSchema {
  pub command: Option<String>,
  pub is_vertical_split: Option<bool>, // dictates split direction [9]
}
