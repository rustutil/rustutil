use serde_json::{json, Value};

use crate::{
    git::clone_else_pull,
    index::{Package, PackageNoVersionsError, PackageNotFoundError, PackageVersionNotFoundError},
};
use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

static SOURCES_DIR: &str = "src";
static BINARY_DIR: &str = "bin";

pub fn add(
    packages: &Vec<Package>,
    versions: &Vec<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    packages
        .into_iter()
        .zip(versions)
        .try_for_each(|(package, version)| add_one(&package, &version))?;
    Ok(())
}

fn add_one(package: &Package, version: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Installing package `{}`...", &package.id);

    if package.versions.as_object().unwrap().is_empty() {
        return Err(Box::new(PackageNoVersionsError(package.id.clone())));
    }

    let branch = &package
        .versions
        .get(&version)
        .ok_or(PackageVersionNotFoundError {
            id: package.id.clone(),
            version: version.to_string(),
        })?;

    println!("Cloning...");
    let mut sources_path = env::current_exe()?;
    sources_path.pop();
    sources_path.push(&SOURCES_DIR);
    fs::create_dir_all(&sources_path)?;

    let mut bin_path = env::current_exe()?;
    bin_path.pop();
    bin_path.push(&BINARY_DIR);
    fs::create_dir_all(&bin_path)?;

    let mut source_path = sources_path.clone();
    source_path.push(&package.id);

    clone_else_pull(&package.repo, &source_path, &branch.as_str().unwrap())?;

    println!("Building...");
    // Build the package and get the output as a bunch of json
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--message-format=json")
        .arg("-q")
        .current_dir(&source_path)
        .output()?;

    // io::stdout().write_all(&output.stdout).unwrap(); // Should add a flag to show this
    io::stderr().write_all(&output.stderr)?;

    println!("Installing...");
    let stdout = String::from_utf8(output.stdout)?;

    // Parse the JSON
    let json: Vec<Value> = stdout
        .split("\n")
        .map(|s| serde_json::from_str(s).unwrap_or_default())
        .collect();

    // Get the path to the compiled binary
    let executable_path = get_executable_path(&json).unwrap();
    let extension = executable_path.extension().unwrap();
    let new_name = format!("{}.{}", &package.id, &extension.to_str().unwrap());
    let new_path = bin_path.join(&new_name);

    fs::rename(&executable_path, &new_path).unwrap();
    println!("Cleaning...");
    Command::new("cargo")
        .arg("clean")
        .current_dir(&source_path)
        .status()?;

    println!("Adding package to binary index...");
    // Map the ID to the path in the binary index
    let mut index_file = bin_path.clone();
    index_file.push("index.json");

    let index_exists = Path::new(&index_file).try_exists()?;
    if !index_exists {
        let empty_object = serde_json::to_string(&json!({}))?;
        fs::write(&index_file, &empty_object)?;
    }

    let index = fs::read_to_string(&index_file)?;
    let mut index: Value = serde_json::from_str(&index).unwrap_or_default();

    index.as_object_mut().unwrap().insert(
        package.id.clone(),
        Value::String(new_path.file_name().unwrap().to_str().unwrap().to_string()),
    );

    fs::write(index_file, serde_json::to_string(&index)?)?;

    println!("Installed `{}`", &package.id);
    println!();
    Ok(())
}

fn get_executable_path(build_out: &Vec<Value>) -> Option<PathBuf> {
    build_out.into_iter().find_map(|json| {
        json.get("executable")
            .and_then(|executable| executable.as_str())
            .map(|path| Path::new(path).to_path_buf())
    })
}

pub fn remove(packages: &Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    packages
        .iter()
        .try_for_each(|package| remove_one(&package))?;
    Ok(())
}

fn remove_one(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Removing package `{}`...", id);

    let mut sources_path = env::current_exe()?;
    sources_path.pop();
    sources_path.push(&SOURCES_DIR);

    let mut bins_path = env::current_exe()?;
    bins_path.pop();
    bins_path.push(&BINARY_DIR);

    let mut index_file = bins_path.clone();
    index_file.push("index.json");

    let index_exists = Path::new(&index_file).try_exists()?;

    if !index_exists {
        return Err(Box::new(PackageNotFoundError(id.to_string())));
    }

    let index = fs::read_to_string(&index_file)?;
    let index: Value = serde_json::from_str(&index).unwrap_or_default();
    let path = index
        .get(id)
        .ok_or(PackageNotFoundError(id.to_string()))?
        .as_str()
        .unwrap();

    println!("Removing from binary index...");
    // Remove the package from the index
    let mut index = index.clone();
    index.as_object_mut().unwrap().remove(id);
    fs::write(index_file, serde_json::to_string(&index)?)?;

    println!("Removing package...");
    // Remove the package
    let mut full_path = bins_path.clone();
    full_path.push(path);

    let mut source_path = sources_path.clone();
    source_path.push(id);

    let source_exists = Path::new(&source_path).try_exists().unwrap();

    if source_exists {
        fs::remove_dir_all(source_path).unwrap();
    }

    fs::remove_file(full_path).unwrap();

    println!("Package removed");
    println!();
    Ok(())
}

pub fn print_versions(package: &Package) {
    for version in package.versions.as_object().unwrap().keys() {
        if version == "latest" {
            continue;
        }
        println!("{}", version);
    }
}
