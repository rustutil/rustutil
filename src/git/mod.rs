use std::path::Path;

use git2::{Repository, ResetType};

pub fn fetch(repo: &git2::Repository, branch: &str) -> Result<(), git2::Error> {
    repo.find_remote("origin")?.fetch(&[branch], None, None)
}

pub fn hard_reset(repo: &git2::Repository, branch: &str) -> Result<(), git2::Error> {
    let remote_branch_ref = repo.refname_to_id(&format!("refs/remotes/origin/{}", branch))?;
    let object = repo.find_object(remote_branch_ref, None)?;
    repo.reset(&object, ResetType::Hard, None)?;
    Ok(())
}

fn pull(repo: git2::Repository, branch: &str) -> Result<(), git2::Error> {
    fetch(&repo, branch)?;
    hard_reset(&repo, branch)?;
    Ok(())
}

pub fn clone_else_pull<P: AsRef<Path>>(
    url: &str,
    into: P,
    branch: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let cloned = Path::try_exists(into.as_ref())?;
    if cloned {
        let repo = Repository::open(into)?;
        pull(repo, branch)?;
    } else {
        Repository::clone(url, into)?;
    };
    Ok(())
}
