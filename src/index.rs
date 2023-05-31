use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{env, error::Error, fmt, fs, path::Path};

use crate::git::clone_else_pull;

static INDEX_DIR: &str = "index";
static PACKAGE_DIR: &str = "packages";
static REPO_URL: &str = "https://github.com/rustutil/.index.git";

#[derive(Serialize, Deserialize)]
pub struct Package {
    pub id: String,
    pub repo: String,
    pub versions: Value, // version: branch
}

impl Package {
    pub fn from_id(id: &str) -> Result<Self, Box<dyn Error>> {
        update()?;
        Package::get(id)
    }
    fn get(id: &str) -> Result<Self, Box<dyn Error>> {
        let mut path = env::current_exe()?;
        path.pop();
        path.push(&INDEX_DIR);
        path.push(&PACKAGE_DIR);
        path.push(id.to_owned() + ".json");

        let path = Path::new(&path);

        let exists = Path::try_exists(&path)?;
        if exists {
            let contents = fs::read_to_string(&path)?;
            let package: Package = serde_json::from_str(&contents)?;
            Ok(package)
        } else {
            Err(Box::new(PackageNotFoundError(id.to_owned())))
        }
    }
    pub fn from_ids(ids: &Vec<&str>) -> Result<Vec<Self>, Box<dyn Error>> {
        update()?;
        ids.iter().map(|id| Package::from_id(id)).collect()
    }
}

#[derive(Debug)]
pub struct PackageNotFoundError(pub String);

impl fmt::Display for PackageNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Package not found: {}", self.0)
    }
}

impl Error for PackageNotFoundError {}

#[derive(Debug)]
pub struct PackageVersionNotFoundError {
    pub id: String,
    pub version: String,
}

impl fmt::Display for PackageVersionNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Version {} not found, run `versions {}` to see avalible versions",
            self.version, self.id
        )
    }
}

impl Error for PackageVersionNotFoundError {}

#[derive(Debug)]
pub struct PackageNoVersionsError(pub String);

impl fmt::Display for PackageNoVersionsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Package `{}` has no versions", self.0)
    }
}

impl Error for PackageNoVersionsError {}

#[derive(Debug)]
pub struct NoBinaryIndex;

impl fmt::Display for NoBinaryIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is no binary index")
    }
}

impl Error for NoBinaryIndex {}

pub fn update() -> Result<(), Box<dyn Error>> {
    let mut path = env::current_exe()?;
    path.pop();
    path.push(INDEX_DIR);
    let path = Path::new(&path);

    clone_else_pull(&REPO_URL, path, "main")?;
    Ok(())
}

pub fn list_ids() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    update()?;
    let mut path = env::current_exe()?;
    path.pop();
    path.push(&INDEX_DIR);
    path.push(&PACKAGE_DIR);

    let files = fs::read_dir(path)?;
    Ok(files
        .map(|file| file.unwrap().path())
        .map(|name| name.file_stem().unwrap().to_str().unwrap().to_owned())
        .collect::<Vec<_>>())
}
