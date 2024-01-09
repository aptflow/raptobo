use raptobo::repository::Repository;
use raptobo::error::RaptoboError;
use raptobo::logger::init_logger;

/// CLI tool apt_check 
/// 
/// This tool parses the metadata of an APT repository.
fn main() -> Result<(), RaptoboError>{
    init_logger();

    let mut repo = Repository::new(
        "http://archive.ubuntu.com/ubuntu",
        "jammy",
        None,
        false,
        false
    );

    repo.load_metadata()?;

    println!("{:?}", repo);

    Ok(())
}
