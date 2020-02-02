use git2;
use git2::Repository;
use log;
use std::error::Error as StdError;
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

pub struct Commit<'repo> {
  pub hash: String,
  pub message: String,
  raw_commit: git2::Commit<'repo>,
}

pub struct CommitIterator<'repo> {
  git_repo: &'repo GitRepo,
  revision_walker: git2::Revwalk<'repo>,
}

type DynError = Box<dyn StdError>;
type DynResult<T> = Result<T, DynError>;

impl<'repo> Iterator for CommitIterator<'repo> {
  type Item = Result<Commit<'repo>, DynError>;

  fn next(&mut self) -> Option<DynResult<Commit<'repo>>> {
    let oid: Option<Result<git2::Oid, git2::Error>> = self.revision_walker.next();

    log::debug!("Iterating {:?}", oid);

    if oid.is_none() {
      return None;
    }

    let oid: DynResult<git2::Oid> = oid.unwrap().map_err(Box::from);
    let oid = oid.map(|oid| {
      return format!("{}", oid);
    });

    let commit = oid
      .map(|oid| {
        return format!("{}", oid);
      })
      .and_then(|oid| {
        return self.git_repo.find_commit_by_id(oid);
      });

    return Some(commit);
  }
}

impl GitRepo {
  pub fn upsert(cid_dir: impl AsRef<Path>, name: &str) -> DynResult<GitRepo> {
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
  pub fn find_commit_by_id(&self, hash: String) -> DynResult<Commit> {
    let commit = self.repo.find_commit(git2::Oid::from_str(&hash)?)?;

    return Ok(Commit {
      hash,
      message: String::from(commit.message().unwrap()),
      raw_commit: commit,
    });
  }

  pub fn absolute_sql_path(&self) -> PathBuf {
    return self.repo_path.join(&self.sql_path);
  }

  pub fn sync_dump(&self, dump: String) -> io::Result<()> {
    // Update content to file in the repo
    return fs::write(self.absolute_sql_path(), dump);
  }

  pub fn commit_dump(&self, message: &str) -> DynResult<()> {
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

  pub fn commit_iterator(&self) -> DynResult<CommitIterator> {
    log::debug!("Getting revwalk...");

    let mut revision_walker = self.repo.revwalk()?;

    // Start walking from HEAD
    revision_walker.push_head()?;

    return Ok(CommitIterator {
      git_repo: self,
      revision_walker: revision_walker,
    });
  }

  pub fn get_dump_at_commit(&self, hash: String) -> DynResult<String> {
    let commit = self.find_commit_by_id(hash)?;
    let commit_tree = commit.raw_commit.tree()?;
    let sql = commit_tree.get_path(Path::new(&self.sql_path))?;
    let sql = sql.to_object(&self.repo)?;
    let sql = sql
      .as_blob()
      .expect(&format!(
        "{} is not a blob",
        self.sql_path.to_str().unwrap()
      ))
      .content();

    return Ok(String::from(std::str::from_utf8(sql)?));
  }
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
    fn it_should_create_repo_when_repo_does_not_exist() {
      let repo_path = String::from("/tmp/test-repo");
      let _dir_cleaner = DirCleaner {
        dir: repo_path.clone(),
      };

      GitRepo::upsert("/tmp", "test-repo").unwrap();

      assert!(PathBuf::from(&repo_path).exists());
      assert!(PathBuf::from(&repo_path).join(".git").exists());
    }
  }

  mod commit {
    use super::*;

    #[test]
    fn it_should_create_initial_commit_on_bare_repo() {}
  }
}
