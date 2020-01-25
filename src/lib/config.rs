use dirs;
use std::path::PathBuf;

pub struct ProjectConfig {
  pub db_uri: String,
  pub project_git_path: PathBuf,
}

pub fn get_project_dir() -> PathBuf {
  let project_dir = dirs::home_dir().unwrap();

  return project_dir.join(".cid");
}
