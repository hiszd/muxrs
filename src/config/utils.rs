pub fn append_path(path: &str, addition: &str) -> String {
  match (path.ends_with("/"), addition.starts_with("/")) {
    // if (true, true) then one of the slashes need to be removed.
    // if either is true then just append
    // if both are false then a slash will need to be added in-between
    (true, true) => path[0..path.len() - 1].to_string() + addition,
    (true, false) | (false, true) => path.to_string() + addition,
    (false, false) => path.to_string() + "/" + addition,
  }
}

pub fn git_path(path: &str) -> Result<String, git2::Error> {
  match git2::Repository::discover(path) {
    Ok(r) => Ok(r.workdir().unwrap().to_string_lossy().to_string()),
    Err(e) => Err(e),
  }
}

pub fn path_string(path: &str) -> String {
  let relative_path = std::path::PathBuf::from(path);
  let absolute_path = std::path::absolute(&relative_path).unwrap();
  absolute_path.to_string_lossy().to_string()
}

pub fn exists_file(path: &str) -> bool { matches!(std::fs::exists(path), Ok(true)) }

pub fn read_file(path: &str) -> Result<String, std::io::Error> { std::fs::read_to_string(path) }

