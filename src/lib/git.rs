use git2;
use git2::Repository;
use log;
use std::error::Error as StdError;
use std::path::Path;
use std::path::PathBuf;

type DynError = Box<dyn StdError>;
type DynResult<T> = Result<T, DynError>;

pub struct GitRepo {
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

    let commit = oid.map(|oid| format!("{}", oid)).and_then(|oid| {
      return self.git_repo.find_commit_by_id(oid);
    });

    return Some(commit);
  }
}

impl GitRepo {
  pub fn new(repo_path: impl AsRef<Path>) -> Result<GitRepo, DynError> {
    let repo = Repository::discover(repo_path.as_ref())?;

    return Ok(GitRepo { repo });
  }

  pub fn upsert(repo_path: impl AsRef<Path>) -> DynResult<GitRepo> {
    let repo_path = PathBuf::from(repo_path.as_ref());

    // This method will automatically:
    // - Creates dir recursively
    // - Creates git repo
    // - Will not do anything if repo is already there (e.g. it'll stop if there's commit)
    let repo = Repository::init(&repo_path)?;

    return Ok(GitRepo { repo });
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

  pub fn commit_file(&self, filepath: impl AsRef<Path>, message: &str) -> DynResult<()> {
    let mut repo_index = self.repo.index()?;
    let filepath = PathBuf::from(filepath.as_ref());

    // Get the old tree first that we will use to simulate `git diff --cached`
    let old_tree = self.repo.find_tree(repo_index.write_tree()?)?;

    repo_index.add_path(filepath.as_ref())?;
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
        filepath
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

  pub fn get_file_content_at_commit(
    &self,
    filepath: impl AsRef<Path>,
    hash: String,
  ) -> DynResult<Vec<u8>> {
    let filepath = PathBuf::from(filepath.as_ref());
    let commit = self.find_commit_by_id(hash)?;
    let commit_tree = commit.raw_commit.tree()?;
    let sql = commit_tree.get_path(filepath.as_ref())?;
    let sql = sql.to_object(&self.repo)?;
    let sql = sql
      .as_blob()
      .expect(&format!("{:?} is not a blob", filepath))
      .content();

    return Ok(Vec::from(sql));
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
    }
    assert!(PathBuf::from(&repo_path).join(".git").exists());
  }

  mod commit {
    use super::*;

    #[test]
    fn it_should_create_initial_commit_on_bare_repo() {}
  }
}
