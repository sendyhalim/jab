use git2::Index;
use git2::Repository;
use std::fs;
use std::io;
use std::path::Path;

struct GitRepo {
  pub initiated: bool,
  pub repo_path: String,
}

impl GitRepo {
  pub fn upsert(cid_dir: impl AsRef<Path>, name: &str) -> io::Result<GitRepo> {
    let repo_path = cid_dir.as_ref().join(name);
    let repo_path = repo_path
      .to_str()
      .expect(&format!("Fail to convert {:?} to &str", repo_path));

    // This method will automatically:
    // - Creates dir recursively
    // - Creates git repo
    // - Will not do anything if repo is already there (e.g. it'll stop if there's commit)
    Repository::init(repo_path).map_err(|git_err| {
      return io::Error::new(io::ErrorKind::Other, git_err.message());
    })?;

    Ok(GitRepo {
      initiated: true,
      repo_path: String::from(repo_path),
    })
  }
}

impl GitRepo {
  pub fn sync_dump(&self, dump: String) {}

  pub fn commit(&self, message: String) {}

  pub fn checkout(&self, hash: String) {}

  pub fn log(&self) {}
}

#[cfg(test)]
mod GitRepoTests {
  use super::*;

  struct DirCleaner {
    dir: String,
  }

  impl Drop for DirCleaner {
    fn drop(&mut self) {
      fs::remove_dir_all(&self.dir);
    }
  }

  mod upsert {
    use super::*;

    #[test]
    fn when_foo() {
      let repo_path = "/tmp/test-repo";
      let dir_cleaner = DirCleaner {
        dir: String::from(repo_path),
      };

      GitRepo::upsert("/tmp", "test-repo");

      assert_eq!(Path::new(repo_path).exists(), true);
    }
  }
}
