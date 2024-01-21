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
    repo.process_files()?;

    log::info!("[apt_check] found {} index files", repo.data.files.len());

    let meta = &repo.metadata.unwrap();

    let components = match &repo.spec.components {
        Some(c) => c,
        None => &meta.components,
    };

    let architectures = &meta.architectures;
    
    for comp in components {
        for arch in architectures {
            log::info!("[apt_check] [{}][{}] {} index files", comp, arch, repo.data.package_indices[comp][arch].len());
            for file in &repo.data.package_indices[comp][arch] {
                log::info!("[apt_check] Index [{}][{}] {}", comp, arch, file);
            }
        }
    }

    Ok(())
}
