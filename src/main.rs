use clap::{Parser, Subcommand, ValueEnum};
use experiments::has_experiments;
use index::{update, Package};

pub mod experiments;
pub mod git;
pub mod index;
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
    Versions { package: String },
}

fn main() {
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

        println!("Warning: You have experiments ({}) enabled. These are experimental and may not work as expected.", experiments_joined);
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
                eprintln!("{}", &packages.err().unwrap());
                return;
            }
            let packages = packages.unwrap();

            let error = packages::add(&packages, &versions, &args.experiments).err();
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
