use git2;
use git2::Repository;
use log;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub struct GitRepo {
  pub initiated: bool,
  pub repo_path: PathBuf,
  sql_path: PathBuf,
  repo: Repository,
}

impl GitRepo {
  pub fn upsert(
    cid_dir: impl AsRef<Path>,
    name: &str,
  ) -> Result<GitRepo, Box<dyn std::error::Error>> {
    let repo_path = PathBuf::from(cid_dir.as_ref().join(name));

    // This method will automatically:
    // - Creates dir recursively
    // - Creates git repo
    // - Will not do anything if repo is already there (e.g. it'll stop if there's commit)
    let repo = Repository::init(&repo_path)?;

    Ok(GitRepo {
      initiated: true,
      repo_path: repo_path,
      sql_path: PathBuf::from("dump.sql"), // Relative to repo_path
      repo,
    })
  }
}

impl GitRepo {
  pub fn absolute_sql_path(&self) -> PathBuf {
    return self.repo_path.join(&self.sql_path);
  }

  pub fn sync_dump(&self, dump: String) -> io::Result<()> {
    // Update content to file in the repo
    return fs::write(self.absolute_sql_path(), dump);
  }

  pub fn commit_dump(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut repo_index = self.repo.index()?;

    // Get the old tree first that we will use to simulate `git diff --cached`
    let old_tree = self.repo.find_tree(repo_index.write_tree()?)?;

    repo_index.add_path(&self.sql_path)?;
    repo_index.write()?;

    log::debug!("Getting the oid for write tree");

    let current_oid = repo_index.write_tree()?;

    log::debug!(
      "Got oid {}, now finding the current tree by oid",
      current_oid
    );

    let current_tree = self.repo.find_tree(current_oid)?;

    if self.repo.is_empty()? {
      log::debug!("Creating initial commit..");

      self.repo.commit(
        Some("HEAD"),
        &self.repo.signature()?, // Author
        &self.repo.signature()?, // Committer
        &message,
        &current_tree,
        &vec![],
      )?;
    } else {
      log::debug!("Finding repo head..");

      let head = self.repo.head()?;
      let head_commit = self.repo.find_commit(head.target().unwrap())?;

      log::debug!(
        "Found repo head: {:?}, adding {:?} to index path",
        head_commit,
        self.sql_path
      );

      // Simulates `git diff --cached`
      let diff = self
        .repo
        .diff_tree_to_index(Some(&old_tree), Some(&repo_index), None)?;

      log::debug!("Diff deltas {:?}", diff.deltas().collect::<Vec<_>>());

      // Only commit if there's changes
      if diff.deltas().len() == 0 {
        return Ok(());
      }

      self.repo.commit(
        Some("HEAD"),
        &self.repo.signature()?, // Author
        &self.repo.signature()?, // Committer
        &message,
        &current_tree,
        &[&head_commit],
      )?;
    }

    return Ok(());
  }

  // pub fn checkout(&self, hash: String) {}

  // pub fn log(&self) {
  // self.repo.

  // }
}

#[cfg(test)]
mod test {
  use super::*;

  struct DirCleaner {
    dir: String,
  }

  impl Drop for DirCleaner {
    fn drop(&mut self) {
      fs::remove_dir_all(&self.dir).unwrap();
    }
  }

  mod upsert {
    use super::*;

    #[test]
    fn when_repo_does_not_existshould_create_repo() {
      let repo_path = String::from("/tmp/test-repo");
      let _dir_cleaner = DirCleaner {
        dir: repo_path.clone(),
      };

      GitRepo::upsert("/tmp", "test-repo").unwrap();

      assert!(PathBuf::from(&repo_path).exists());
      assert!(PathBuf::from(&repo_path).join(".git").exists());
    }
  }
}
