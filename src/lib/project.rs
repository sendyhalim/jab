use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub struct ProjectConfig {
  pub name: String,
  pub project_dir: PathBuf,
}

pub struct Project {
  name: String,
  project_dir: PathBuf,
  repo_path: PathBuf,
  sql_path: PathBuf,
}

impl Project {
  pub fn open(config: ProjectConfig) -> Project {
    return Project {
      project_dir: config.project_dir.clone(),
      name: config.name.clone(),
      sql_path: PathBuf::from("dump.sql"),
      repo_path: config.project_dir.join(config.name),
    };
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

  fn default_sql_path() -> PathBuf {
    return PathBuf::from("dump.sql");
  }

  pub fn absolute_sql_path(&self) -> PathBuf {
    return self.repo_path.join(&self.sql_path);
  }

  pub fn sync_dump(&self, dump: Vec<u8>) -> io::Result<()> {
    // Update content to file in the repo
    return fs::write(self.absolute_sql_path(), dump);
  }
}
