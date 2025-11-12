use clap::Parser;
pub mod config;
pub mod tmux;

use config::utils::path_string;

pub const ABOUT: &str = r#"
muxrs quickly launches and configures complex, reproducible tmux development environments.
It reads muxrs.json from the git root or XDG_CONFIG_HOME directory and creates a new tmux session
with custom windows, panes, directories, and startup commands, much like tmuxinator.
"#;

// NOTE: The proper usage of arguments is the following:
// If "git "is specified it will either look within the path specified for a git repo, or look within
// the current working directory for a git repo.
// If "git" or "path" is specified and "config" is ALSO specified it will determine the working path from
// the "path" arg, maybe finding the base git directory if "git" is also specified but use the config from the "config" arg
#[derive(Parser, Clone)]
#[command(next_line_help = true)]
#[command(version, about, long_about = ABOUT)]
pub struct Args {
  /// The path to the configuration file (muxrs.json)
  #[arg(short, long)]
  config: Option<String>,
  /// Go to the root of the git repository for the config file
  // TODO: change this flag so that git is assumed by default
  #[arg(short = 'g', long)]
  no_git: bool,
  /// Turn debugging information on
  #[arg(short, long)]
  debug: bool,
  /// Turn off falling back to the default config located in the XDG_CONFIG_HOME directory
  #[arg(short = 'f', long)]
  no_fallback: bool,
  /// Attach to the tmux session after it is created
  #[arg(short = 'a', long)]
  no_attach: bool,
  /// Check the config file for validity instead of creating the session
  #[arg(short, long)]
  verify: bool,
  /// Populate the config file with default values
  #[arg(short, long)]
  populate: bool,
  /// The path to the working directory
  #[arg(value_name = "PATH")]
  path: Option<String>,
}

fn main() {
  let args = {
    let mut a = Args::parse();
    if a.path.is_some() {
      a.path = path_string(a.path.as_ref().unwrap().as_str()).into();
    }
    a
  };

  if args.debug {
    tracing_subscriber::fmt()
      .with_env_filter(tracing_subscriber::EnvFilter::new("INFO"))
      .init();
  } else {
    tracing_subscriber::fmt()
      .with_env_filter(tracing_subscriber::EnvFilter::new("ERROR"))
      .init();
  }

  if args.verify {
    match config::get_config(args.clone()) {
      Ok(_) => {
        println!("Config is valid");
        return;
      }
      Err(e) => {
        println!("Invalid config: {:?}", e.to_string());
        return;
      }
    };
  }

  // NOTE: This populates a new config file at the specified config location with the default values
  if args.populate {
    if args.config.is_none() {
      println!("Please specify a config file");
      return;
    }
    let config = config::schema::ConfigSchema::default();
    let json = serde_json::to_string_pretty(&config).unwrap();
    match config::utils::write_file(&args.config.unwrap(), json) {
      Ok(_) => {
        println!("Config file populated");
        return;
      }
      Err(e) => {
        tracing::error!("Failed to create config file: {}", e.to_string());
        return;
      }
    }
  }

  // NOTE: Step 1: Get the config
  let config = match config::get_config(args.clone()) {
    Ok(c) => c,
    Err(e) => {
      tracing::error!("{:?}", e.to_string());
      return;
    }
  };

  let windows = tmux::query::get_windows(&config.session);
  tracing::info!("{:?}", windows);

  // NOTE: Step 2: Ensure a session does not exist already with the name specified in the config
  // NOTE: Step 3: Create the session
  let session = match tmux::Session::new(config.session.clone()) {
    Ok(s) => s,
    Err(e) => match e {
      tmux::TmuxError::CouldNotCreateSession(_) => {
        if !args.no_attach {
          println!(
            "A session with the name {} already exists. Attaching now.",
            config.session.name
          );
          tmux::command::attach(&config.session);
          return;
        }
        println!("A session with the name {} already exists.", config.session.name);
        return;
      }
      _ => {
        tracing::error!("{:?}", e.to_string());
        return;
      }
    },
  };

  let mut active_windows = Vec::new();

  // NOTE: Step 4: Configure each window and pane
  for (i, window) in config.windows.iter().enumerate() {
    let starting_dir = match window.starting_dir.clone() {
      Some(d) => d,
      None => config
        .session
        .starting_dir
        .clone()
        .unwrap_or(".".to_string()),
    };
    if i == 0 {
      match session.respawn_window("0", &starting_dir) {
        Ok(_) => {
          tracing::info!("Created new window for index {}", i);
        }
        Err(e) => {
          tracing::error!("{:?}", e.to_string());
          return;
        }
      }
      match session.rename_window(0, &window.name) {
        Ok(_) => {
          tracing::info!("Created new window for index {}", i);
        }
        Err(e) => {
          tracing::error!("{:?}", e.to_string());
          return;
        }
      }
    } else {
      // NOTE: New window starts at the "starting_dir" provided in the window config.
      // If no "starting_dir" is provided, the session's "starting_dir" is used
      match session.new_window(window) {
        Ok(_) => {
          tracing::info!("Created new window for index {}", i);
        }
        Err(e) => {
          tracing::error!("{:?}", e.to_string());
          return;
        }
      }
    }
    if window.panes.is_some() {
      for (j, pane) in window.panes.clone().unwrap().iter().enumerate() {
        // NOTE: Handle the default pane and execute the command in that pane first
        if j != 0 {
          // NOTE: Handle the other panes
          match session.split_window(
            &window.name,
            window.starting_dir.clone(),
            pane.is_vertical_split.unwrap_or(false),
          ) {
            Ok(_) => {
              tracing::info!("Split window");
            }
            Err(e) => {
              tracing::error!("{:?}", e.to_string());
              return;
            }
          }
        }
        // NOTE: Execute the command in the pane
        if pane.command.is_some() {
          match session.send_keys(window, &pane.command.clone().unwrap(), None) {
            Ok(_) => {
              tracing::info!("Executed command");
            }
            Err(e) => {
              tracing::error!("{:?}", e.to_string());
              return;
            }
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
          tracing::info!("Focused window: {}", active_windows[0].name.clone());
        }
        Err(e) => {
          tracing::error!("{:?}", e.to_string());
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
  if !args.no_attach {
    println!("Attaching to session: {}", &config.session.name);
    tmux::command::attach(&config.session);
  }
}

#[cfg(test)]
mod tests {
  use crate::config::schema::{
    ConfigSchema,
    PaneSchema,
    WindowSchema,
  };

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
        panes: Some(vec![PaneSchema {
          command: Some("nvim".to_string()),
          is_vertical_split: None,
        }]),
      }],
    );

    let c = serde_json::to_string_pretty(&t).unwrap();
    println!("{}", c);
  }
}
