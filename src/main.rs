use clap::{Parser, Subcommand};
use index::{update, Package};

pub mod git;
pub mod index;
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
        /// Package ID
        package: String,
        /// Package version, latest if not specified
        version: Option<String>,
    },
    /// Removes a package
    Remove {
        // Package ID
        package: String,
    },
    /// Updates the index
    Update,
    /// Lists versions of a package
    Versions { package: String },
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::Add {
            package: id,
            version,
        } => {
            let package = Package::from_id(id.to_string()).expect("Package not found");
            let version = &version.clone().unwrap_or("latest".to_owned());
            packages::add(&package, version).unwrap();
        }
        Commands::Remove { package: id } => {
            packages::remove(&id).unwrap();
        }
        Commands::Update => {
            update().unwrap();
        }
        Commands::Versions { package: id } => {
            let package = Package::from_id(id.to_string()).expect("Package not found");
            packages::versions(&package);
        }
    };
}
