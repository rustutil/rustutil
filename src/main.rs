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
    Versions { package: String },
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::Add { ids } => {
            let packages = Package::from_ids(ids);
            if packages.is_err() {
                eprintln!("{}", &packages.err().unwrap());
                return;
            }
            let packages = packages.unwrap();

            let versions = ids
                .iter()
                .map(|i| i.split("@").nth(1).unwrap_or("latest"))
                .collect::<Vec<&str>>();

            let error = packages::add(&packages, &versions).err();
            if error.is_some() {
                eprintln!("{}", error.unwrap());
            }
        }
        Commands::Remove { ids } => {
            let error = packages::remove(ids).err();
            if error.is_some() {
                eprintln!("{}", error.unwrap());
            }
        }
        Commands::Update => {
            let error = update().err();
            if error.is_some() {
                eprintln!("{}", error.unwrap());
            }
        }
        Commands::Versions { package: id } => {
            let package = Package::from_id(id).expect("Package not found");
            packages::print_versions(&package);
        }
    };
}
