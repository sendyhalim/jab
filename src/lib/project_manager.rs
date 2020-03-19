use std::path::Path;

use crate::config::JabConfig;
use crate::project::Project;
use crate::types::ResultDynError;

pub struct CreateProjectInput<'a> {
  pub project_dir: &'a Path,
  pub project_name: &'a str,
  pub db_uri: &'a str,
}

pub struct OpenProjectInput<'a> {
  pub project_dir: &'a Path,
  pub project_name: &'a str,
  pub db_uri: &'a str,
}

pub trait ProjectManager {
  fn bootstrap() -> ResultDynError<()>;
  fn new(jab_config: JabConfig) -> Self;
  fn create_project(&mut self, input: &CreateProjectInput) -> ResultDynError<Project>;
  fn open_project(&self, input: &OpenProjectInput) -> ResultDynError<Project>;
  fn get_project_names(&self) -> ResultDynError<Vec<&str>>;
  // fn open_project()
}
