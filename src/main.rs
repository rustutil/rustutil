use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use experiments::has_experiments;
use index::{update, Package};
use input::prompt;
use log::{error, warn};
use logger::init;
use serde_json::json;
use url::Url;

pub mod experiments;
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

    /// Enables some experiments
    #[arg(short = 'e', long = "experiment-enable", value_enum)]
    experiments: Option<Vec<Experiment>>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Experiment {
    /// Enables the target cache, this saves the target directory to /targets.
    /// This saves rebuilding a packages depencencies when updating.
    /// The folder is deleted when the package is removed.
    TargetCache,
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

    if has_experiments(&args.experiments) {
        let experiments_joined = &args
            .experiments
            .as_ref()
            .unwrap()
            .into_iter()
            .map(|e| e.to_possible_value().unwrap().get_name().to_owned())
            .collect::<Vec<String>>()
            .join(", ");

        warn!(
            "You have experiments ({}) enabled. These are experimental and may not work as expected.",
            experiments_joined.blue()
        );
    }

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

            let error = packages::add(&packages, &versions, &args.experiments).err();
            if error.is_some() {
                error!("{}", error.unwrap());
            }
        }
        Commands::Remove { ids } => {
            let error = packages::remove(&ids, &args.experiments).err();
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
            packages::print_versions(&package);
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
                    // TODO: Check if it is a git repository
                    if Url::parse(&repo).is_err() {
                        error!("Invalid repository URL");
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
    };
}
