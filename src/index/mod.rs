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
    pub fn from_id(id: String) -> Result<Self, Box<dyn Error>> {
        update()?;
        let mut path = env::current_exe()?;
        path.pop();
        path.push(&INDEX_DIR);
        path.push(&PACKAGE_DIR);
        path.push(id.clone() + ".json");

        let path = Path::new(&path);

        let exists = Path::try_exists(&path)?;
        if exists {
            let contents = fs::read_to_string(&path)?;
            let package: Package = serde_json::from_str(&contents)?;
            Ok(package)
        } else {
            Err(Box::new(PackageNotFoundError(id)))
        }
    }
}

#[derive(Debug)]
struct PackageNotFoundError(String);

impl fmt::Display for PackageNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Package not found: {}", self.0)
    }
}

impl Error for PackageNotFoundError {}

pub fn update() -> Result<(), Box<dyn Error>> {
    let mut path = env::current_exe()?;
    path.pop();
    path.push(INDEX_DIR);
    let path = Path::new(&path);

    clone_else_pull(&REPO_URL, path, "main")?;
    Ok(())
}
