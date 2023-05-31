use log::info;
use serde_json::{json, map::Keys, Value};

use crate::{
    experiments::has_experiment,
    git::clone_else_pull,
    index::{
        NoBinaryIndex, Package, PackageNoVersionsError, PackageNotFoundError,
        PackageVersionNotFoundError,
    },
    Experiment,
};
use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};

static SOURCES_DIR: &str = "src";
static BINARY_DIR: &str = "bin";
static TARGETS_DIR: &str = "targets";

pub fn add(
    packages: &Vec<Package>,
    versions: &Vec<&str>,
    experiments: &Option<Vec<Experiment>>,
) -> Result<(), Box<dyn std::error::Error>> {
    packages
        .into_iter()
        .zip(versions)
        .try_for_each(|(package, version)| add_one(&package, &version, &experiments))?;
    Ok(())
}

fn clone(
    package: &Package,
    branch: &str,
    source_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Cloning");
    clone_else_pull(&package.repo, &source_path, &branch)?;
    Ok(())
}

fn target_cache_copy(
    package: &Package,
    source_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(target: "target-cache", "Copying Cache");
    let mut targets_path = env::current_exe()?;
    targets_path.pop();
    targets_path.push(&TARGETS_DIR);
    fs::create_dir_all(&targets_path)?;

    let mut target_path = targets_path.clone();
    target_path.push(&package.id);

    let mut cargo_target_path = source_path.clone();
    cargo_target_path.push("target");

    if Path::try_exists(&target_path)? {
        fs::rename(target_path, cargo_target_path)?;
    }

    Ok(())
}

fn build(
    package: &Package,
    source_path: &PathBuf,
    experiments: &Option<Vec<Experiment>>,
) -> Result<Output, Box<dyn std::error::Error>> {
    if has_experiment(experiments, &Experiment::TargetCache) {
        target_cache_copy(&package, &source_path)?;
    }

    info!("Building");

    // Build the package and get the output as a bunch of json
    // If target-cache is enabled, it will use that instead of
    // rebuilding the package and its depenencies from scratch
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--message-format=json")
        .arg("-q")
        .current_dir(&source_path)
        .output()?;

    // io::stdout().write_all(&output.stdout).unwrap(); // Should add a flag to show this
    io::stderr().write_all(&output.stderr)?;

    Ok(output)
}

fn cache(package: &Package, source_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!(target: "target-cache", "Caching");

    let mut targets_path = env::current_exe()?;
    targets_path.pop();
    targets_path.push(&TARGETS_DIR);
    fs::create_dir_all(&targets_path)?;

    let mut target_path = targets_path.clone();
    target_path.push(&package.id);

    let mut cargo_target_path = source_path.clone();
    cargo_target_path.push("target");

    if Path::try_exists(&target_path)? {
        fs::remove_dir_all(&target_path)?;
    }
    fs::rename(&cargo_target_path, &target_path)?;
    Ok(())
}
fn clean(source_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!("Cleaning");
    Command::new("cargo")
        .arg("clean")
        .current_dir(&source_path)
        .status()?;
    Ok(())
}

fn install(
    package: &Package,
    bin_path: &PathBuf,
    output: &Output,
) -> Result<String, Box<dyn std::error::Error>> {
    info!("Installing");
    let stdout = String::from_utf8(output.to_owned().stdout)?;

    // Parse the JSON
    let json: Vec<Value> = stdout
        .split("\n")
        .map(|s| serde_json::from_str(s).unwrap_or_default())
        .collect();

    // Get the path to the compiled binary
    let executable_path = get_executable_path(&json).unwrap();
    let out_extension = executable_path.extension().unwrap();
    let out_name = format!("{}.{}", &package.id, &out_extension.to_str().unwrap());
    let out_path = bin_path.join(&out_name);

    fs::rename(&executable_path, &out_path)?;
    Ok(out_name)
}

fn add_one(
    package: &Package,
    version: &str,
    experiments: &Option<Vec<Experiment>>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Installing package `{}`", &package.id);

    if package.versions.as_object().unwrap().is_empty() {
        return Err(Box::new(PackageNoVersionsError(package.id.clone())));
    }

    let branch = &package
        .versions
        .get(&version)
        .ok_or(PackageVersionNotFoundError {
            id: package.id.clone(),
            version: version.to_string(),
        })?
        .as_str()
        .unwrap();

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

    clone(&package, &branch, &source_path)?;
    let output = build(&package, &source_path, &experiments)?;
    let out_name = install(package, &bin_path, &output)?;

    if has_experiment(experiments, &Experiment::TargetCache) {
        cache(&package, &source_path)?;
    } else {
        clean(&source_path)?;
    }
    info!("Adding to binary index");
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

    index
        .as_object_mut()
        .unwrap()
        .insert(package.id.clone(), Value::String(out_name));

    fs::write(index_file, serde_json::to_string(&index)?)?;

    info!("Installed `{}`", &package.id);
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

pub fn remove(
    packages: &Vec<String>,
    experiments: &Option<Vec<Experiment>>,
) -> Result<(), Box<dyn std::error::Error>> {
    packages
        .iter()
        .try_for_each(|package| remove_one(&package, experiments))?;
    Ok(())
}

fn remove_bin(index_file: &PathBuf, id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let index = fs::read_to_string(&index_file)?;
    let index: Value = serde_json::from_str(&index).unwrap_or_default();
    let path = index
        .get(id)
        .ok_or(PackageNotFoundError(id.to_string()))?
        .as_str()
        .unwrap();

    info!("Removing from binary index");
    // Remove the package from the index
    let mut index = index.clone();
    index.as_object_mut().unwrap().remove(id);
    fs::write(index_file, serde_json::to_string(&index)?)?;
    Ok(path.to_owned())
}

fn remove_pkg(
    bins_path: &PathBuf,
    sources_path: &PathBuf,
    executable_path: &String,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Removing package");
    // Remove the package
    let mut full_path = bins_path.clone();
    full_path.push(&executable_path);

    let mut source_path = sources_path.clone();
    source_path.push(&id);

    let source_exists = Path::new(&source_path).try_exists().unwrap();

    if source_exists {
        fs::remove_dir_all(&source_path)?;
    }

    fs::remove_file(full_path)?;
    Ok(())
}

fn remove_one(
    id: &str,
    experiments: &Option<Vec<Experiment>>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Removing package `{}`", id);

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

    let executable_path = remove_bin(&index_file, &id)?;
    remove_pkg(&bins_path, &sources_path, &executable_path, &id)?;

    if has_experiment(experiments, &Experiment::TargetCache) {
        let mut targets_path = env::current_exe()?;
        targets_path.pop();
        targets_path.push(&TARGETS_DIR);
        fs::create_dir_all(&targets_path)?;

        let mut target_path = targets_path.clone();
        target_path.push(&id);

        if Path::try_exists(&target_path)? {
            fs::remove_dir_all(&target_path)?;
        }
    }

    info!("Package removed");
    println!();
    Ok(())
}

pub fn versions(package: &Package) -> Keys {
    package.versions.as_object().unwrap().keys()
}

pub fn list() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut index_file = env::current_exe()?;
    index_file.pop();
    index_file.push(&BINARY_DIR);
    index_file.push("index.json");

    let index_exists = Path::new(&index_file).try_exists()?;

    if !index_exists {
        return Err(Box::new(NoBinaryIndex));
    }

    let index = fs::read_to_string(&index_file)?;
    let index: Value = serde_json::from_str(&index).unwrap_or_default();
    Ok(index
        .as_object()
        .unwrap()
        .keys()
        .map(|x| x.to_owned())
        .collect::<Vec<String>>())
}
