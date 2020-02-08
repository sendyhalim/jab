use std::path::Path;

use crate::config::CidConfig;
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
  fn new(cid_config: CidConfig) -> Self;
  fn create_project(&mut self, input: &CreateProjectInput) -> ResultDynError<Project>;
  fn open_project(&self, input: &OpenProjectInput) -> ResultDynError<Project>;

  // fn open_project()
}
