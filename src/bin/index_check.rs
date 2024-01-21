use raptobo::error::RaptoboError;
use raptobo::logger::init_logger;
use raptobo::package::PackageMetadata;
use raptobo::utils::{download, download_gz, download_xz};
use clap::{arg, command};
use std::cmp::max;

fn main() -> Result<(), RaptoboError> {
    init_logger();

    let matches = command!()
        .arg(arg!([url] "URL of the package index"))
        .get_matches();

    if let Some(url) = matches.get_one::<String>("url") {
        let content = if url.to_lowercase().ends_with("xz") {
            download_xz(url)?
        } else if url.to_lowercase().ends_with("gz") {
            download_gz(url)?
        } else {
            download(url)?
        };

        let packages = PackageMetadata::parse(content)?;

        log::info!("Found {} packages.", packages.len());

        if !packages.is_empty() {
            let len = max(10, packages.len());
            for package in &packages[..len] {
                log::info!("{:?}", package);
            }
        }
    } else {
        return Err(RaptoboError::new("Parameter url is required!"));
    }

    Ok(())
}
