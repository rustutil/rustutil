use clap::{Parser, Subcommand};
use index::{update, Package};
use log::error;
use logger::init;

pub mod git;
pub mod index;
pub mod logger;
pub mod packages;

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
    /// Updates the index
    Update,
}

fn main() {
    init().expect("Failed to init logger");

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
            IndexCommands::Update => {
                let error = update().err();
                if error.is_some() {
                    error!("{}", error.unwrap());
                }
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
