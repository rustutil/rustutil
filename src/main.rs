use clap::{Parser, Subcommand};
use index::{update, Package};
use input::prompt;
use log::error;
use logger::init;
use serde_json::json;

pub mod git;
pub mod index;
pub mod input;
pub mod logger;
pub mod packages;
pub mod repo;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Adds a package
    Add {
        /// Package ID(s), specify a version by adding @version. Example: "package@1.0.0", will automatically add the latest version if not specified
        ids: Vec<String>,
    },
    /// Removes a package
    Remove {
        // Package ID(s)
        ids: Vec<String>,
    },
    /// Updates the index
    Update,
    /// Lists versions of a package
    Versions { id: String },
    /// Lists the installed packages
    List,
    /// Commands related to the index
    Index {
        #[command(subcommand)]
        command: IndexCommands,
    },
}

#[derive(Subcommand, Debug)]
enum IndexCommands {
    /// Lists all avalible packages. WARNING: This is slow and will flood your terminal!
    List,
    /// Makes a package manifest
    CreatePackage,
}

fn main() {
    init().unwrap();

    let args = Args::parse();

    match &args.command {
        Commands::Add { ids: id_versions } => {
            let split = id_versions.iter().map(|x| x.split("@"));
            let ids = split
                .clone()
                .map(|mut x| x.nth(0).unwrap())
                .collect::<Vec<&str>>();
            let versions = split
                .clone()
                .map(|mut x| x.nth(1).unwrap_or("latest"))
                .collect::<Vec<&str>>();

            let packages = Package::from_ids(&ids);
            if packages.is_err() {
                error!("{}", &packages.err().unwrap());
                return;
            }
            let packages = packages.unwrap();

            let error = packages::add(&packages, &versions).err();
            if error.is_some() {
                error!("{}", error.unwrap());
            }
        }
        Commands::Remove { ids } => {
            let error = packages::remove(&ids).err();
            if error.is_some() {
                error!("{}", error.unwrap());
            }
        }
        Commands::Update => {
            let error = update().err();
            if error.is_some() {
                error!("{}", error.unwrap());
            }
        }
        Commands::Versions { id } => {
            let package = Package::from_id(&id);
            if package.is_err() {
                error!("{}", &package.err().unwrap());
                return;
            }
            let package = package.unwrap();
            let versions = packages::versions(&package);
            for version in versions {
                println!("{}", version);
            }
        }
        Commands::Index { command } => match command {
            IndexCommands::List => {
                let ids = index::list_ids();

                if ids.is_err() {
                    error!("{}", &ids.err().unwrap());
                    return;
                }

                let ids = ids.unwrap();

                for id in ids {
                    println!("{}", id);
                }
            }
            IndexCommands::CreatePackage => {
                let mut id;
                let mut repo;
                let mut latest;
                'id: loop {
                    id = prompt("Package ID").unwrap();
                    if id.contains(" ") || id.is_empty() || !id.is_ascii() {
                        error!("Package ID may not contain spaces, be empty or non-ascii");
                    } else {
                        break 'id;
                    }
                }
                'repo: loop {
                    repo = prompt("Repository").unwrap();
                    // TODO: Smaller Validation, this makes binary 0.24 MiB Larger
                    //url::Url::parse(&repo).is_err()
                    if false {
                        error!("Repository must be a valid URL")
                    } else {
                        break 'repo;
                    }
                }
                'latest: loop {
                    latest = prompt("Main branch").unwrap();
                    if latest.contains(" ") || latest.is_empty() || !latest.is_ascii() {
                        error!("Branch name may not contain spaces, be empty or non-ascii");
                    } else {
                        break 'latest;
                    }
                }
                let package: Package = Package {
                    id,
                    repo,
                    versions: json!({ "latest": latest }),
                };
                let string = serde_json::to_string(&package).unwrap();
                println!("Package JSON\n{}", string);
            }
        },
        Commands::List => {
            let list = packages::list();

            if list.is_err() {
                error!("{}", &list.err().unwrap());
                return;
            }

            let list = list.unwrap();

            for package in list {
                println!("{}", package);
            }
        }
    };
}
