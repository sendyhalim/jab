use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub struct ProjectConfig {
  pub repo_path: PathBuf,
  pub sql_path: PathBuf,
}

pub struct Project {
  pub config: ProjectConfig,
}

impl Project {
  fn default_sql_path() -> PathBuf {
    return PathBuf::from("dump.sql");
  }

  pub fn absolute_sql_path(&self) -> PathBuf {
    return self.config.repo_path.join(&self.config.sql_path);
  }

  pub fn sync_dump(&self, dump: Vec<u8>) -> io::Result<()> {
    // Update content to file in the repo
    return fs::write(self.absolute_sql_path(), dump);
  }
}
