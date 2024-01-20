use clap::Parser;
use raptobo::error::RaptoboError;
use raptobo::logger::init_logger;
use raptobo::repository::RepositorySpec;

/// CLI tool apt_check
///
/// This tool parses the metadata of an APT repository.
fn main() -> Result<(), RaptoboError> {
    init_logger();

    let spec = RepositorySpec::parse();

    let mut repo = spec.to_repo();

    repo.load_metadata()?;

    println!("{:?}", repo);

    Ok(())
}
