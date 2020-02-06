use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use crate::git::GitRepo;
use crate::types::ResultDynError;

pub struct CreateInput {
  pub project_name: String,
  pub project_dir: PathBuf,
  pub db_uri: String,
}

pub struct OpenInput {
  pub project_dir: PathBuf,
  pub project_name: String,
  pub db_uri: String,
}

pub struct Project {
  name: String,
  project_dir: PathBuf,
  repo_path: PathBuf,
  sql_path: PathBuf,
  db_uri: String,
}

impl Project {
  pub fn create(input: CreateInput) -> ResultDynError<Project> {
    let repo_path = input.project_dir.join(&input.project_name);
    let _repo = GitRepo::upsert(repo_path)?;

    let project = Project::open(OpenInput {
      project_dir: input.project_dir,
      project_name: input.project_name,
      db_uri: input.db_uri,
    })?;

    return Ok(project);
  }

  pub fn open(input: OpenInput) -> ResultDynError<Project> {
    // TODO: Validate if project exists
    return Ok(Project {
      db_uri: input.db_uri,
      project_dir: input.project_dir.clone(),
      name: input.project_name.clone(),
      sql_path: Project::default_sql_path(),
      repo_path: input.project_dir.join(input.project_name),
    });
  }

  fn default_sql_path() -> PathBuf {
    return PathBuf::from("dump.sql");
  }
}

impl Project {
  pub fn db_uri(&self) -> &str {
    return &self.db_uri;
  }

  pub fn project_dir(&self) -> &Path {
    return self.project_dir.as_ref();
  }

  pub fn repo_path(&self) -> &Path {
    return self.repo_path.as_ref();
  }

  pub fn sql_path(&self) -> &Path {
    return self.sql_path.as_ref();
  }

  pub fn name(&self) -> &str {
    return &self.name;
  }

  pub fn absolute_sql_path(&self) -> PathBuf {
    return self.repo_path.join(&self.sql_path);
  }

  pub fn sync_dump(&self, dump: Vec<u8>) -> io::Result<()> {
    // Update content to file in the repo
    return fs::write(self.absolute_sql_path(), dump);
  }
}
