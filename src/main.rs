use clap::Parser;

pub mod config;
pub mod tmux;

pub const ABOUT: &str = r#"
muxrs quickly launches and configures complex, reproducible tmux development environments.
It reads muxrs.json from the git root or XDG_CONFIG_HOME directory and creates a new tmux session
with custom windows, panes, directories, and startup commands, much like tmuxinator.
"#;

// TODO: Include an option to disable config fallback and fail if not found
// TODO: Include an option for checking a specified config file
#[derive(Parser, Clone)]
#[command(next_line_help = true)]
#[command(version, about, long_about = ABOUT)]
pub struct Args {
  /// The path to the configuration file (muxrs.json)
  #[arg(short, long)]
  config: Option<String>,
  /// Go to the root of the git repository for the config file
  #[arg(short, long)]
  git: bool,
  /// Turn debugging information on
  // TODO:
  #[arg(short, long)]
  debug: bool,
  /// Turn off falling back to the default config located in the XDG_CONFIG_HOME directory
  // TODO:
  #[arg(short, long)]
  no_fallback: bool,
  #[arg(short, long)]
  attach: bool,
}

fn main() {
  let args = {
    let mut a = Args::parse();
    if a.config.is_some() {
      let relative_path = std::path::PathBuf::from(a.config.unwrap());
      let absolute_path = std::path::absolute(&relative_path).unwrap();
      a.config = Some(absolute_path.to_string_lossy().to_string());
    }
    a
  };

  // NOTE: Step 1: Get the config
  let config = match config::get_config(args.clone()) {
    Ok(c) => c,
    Err(e) => {
      println!("{:?}", e.to_string());
      return;
    }
  };

  let windows = tmux::query::get_windows(&config.session);
  println!("{:?}", windows);

  // NOTE: Step 2: Ensure a session does not exist already with the name specified in the config
  // NOTE: Step 3: Create the session
  let session = match tmux::Session::new(config.session.clone()) {
    Ok(s) => s,
    Err(e) => {
      println!("{:?}", e.to_string());
      return;
    }
  };

  let mut active_windows = Vec::new();

  // NOTE: Step 4: Configure each window and pane
  for (i, window) in config.windows.iter().enumerate() {
    if i == 0 {
      match session.rename_window(i, &window.name) {
        Ok(_) => {
          println!("Renamed first window");
        }
        Err(e) => {
          println!("{:?}", e.to_string());
          return;
        }
      }
    } else {
      match session.new_window(window) {
        Ok(_) => {
          println!("Created new window");
        }
        Err(e) => {
          println!("{:?}", e.to_string());
          return;
        }
      }
    }
    for (j, pane) in window.panes.iter().enumerate() {
      // NOTE: Handle the default pane and execute the command in that pane first
      if j != 0 {
        // NOTE: Handle the other panes
        match session.split_window(&window.name, pane.is_vertical_split.unwrap_or(false)) {
          Ok(_) => {
            println!("Split window");
          }
          Err(e) => {
            println!("{:?}", e.to_string());
            return;
          }
        }
      }
      // NOTE: Execute the command in the pane
      if pane.command.is_some() {
        match session.send_keys(window, &pane.command.clone().unwrap(), None) {
          Ok(_) => {
            println!("Executed command");
          }
          Err(e) => {
            println!("{:?}", e.to_string());
            return;
          }
        }
      }
    }
    if let Some(true) = window.set_active {
      active_windows.push(window.clone());
    }
  }
  match active_windows.len() {
    x if x > 0 => {
      match session.select_window(&active_windows[0]) {
        Ok(_) => {
          println!("Focused window: {}", active_windows[0].name.clone());
        }
        Err(e) => {
          println!("{:?}", e.to_string());
          return;
        }
      }
      if x > 1 {
        println!("Warning: More than one window is set to be active");
        println!("setting focus for the first one: {}", active_windows[0].name.clone());
      }
    }
    _ => {}
  }
  if args.attach {
    tmux::command::attach(&config.session).unwrap();
  }
}

#[cfg(test)]
mod tests {
  use crate::config::schema::{ConfigSchema, PaneSchema, WindowSchema};

  #[test]
  fn print_config() {
    let t = ConfigSchema::new(
      crate::config::schema::SessionSchema {
        name: "nvim".to_string(),
        starting_dir: Some("/etc/nothing".to_string()),
      },
      vec![WindowSchema {
        name: "nvim".to_string(),
        starting_dir: Some("/etc/nothing".to_string()),
        set_active: None,
        panes: vec![PaneSchema {
          command: Some("nvim".to_string()),
          is_vertical_split: None,
        }],
      }],
    );

    let c = serde_json::to_string_pretty(&t).unwrap();
    println!("{}", c);
  }
}
