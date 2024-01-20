use crate::error::RaptoboError;
use crate::utils::{
    download, parse_metadata, stanza_files, stanza_list, stanza_opt_value, stanza_text,
    stanza_value, File,
};
use chrono::NaiveDateTime;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct RepositorySpec {
    /// Is the repository using a flat layout?
    #[arg(short, long, default_value_t = false)]
    pub flat: bool,
    /// Is the repository a source repository?
    #[arg(short, long, default_value_t = false)]
    pub source: bool,
    /// URI of the repository root
    #[arg(short = 'r', long = "repository", default_value_t = String::from("http://archive.ubuntu.com/ubuntu"))]
    pub uri: String,
    /// Name of the distribution, or path to root in case of flat repository
    #[arg(short, long, default_value_t = String::from("jammy"))]
    pub distribution: String,
    /// Components to use
    #[arg(short, long)]
    pub components: Option<Vec<String>>,
}

impl RepositorySpec {
    pub fn to_repo(self) -> Repository {
        Repository {
            spec: self,
            metadata: None,
        }
    }
}

#[derive(Debug)]
pub struct RepositoryMetadata {
    pub architectures: Vec<String>,
    pub components: Vec<String>,
    pub description: String,
    pub origin: Option<String>,
    pub label: Option<String>,
    pub version: String,
    pub suite: Option<String>,
    pub codename: String,
    pub date: NaiveDateTime,
    pub md5sum: Vec<File>,
    pub sha1: Vec<File>,
    pub sha256: Vec<File>,
}

impl RepositoryMetadata {
    pub fn new(content: Vec<String>) -> Result<RepositoryMetadata, RaptoboError> {
        let data = parse_metadata(content)?;

        // search right stanza
        let stanza = data
            .into_iter()
            .find(|d| d.contains_key("Codename"))
            .ok_or(RaptoboError::new(
                "[RepositoryMetadata] Codename not found!",
            ))?;

        let date = stanza_value("Date", &stanza)?;
        let date = NaiveDateTime::parse_from_str(&date, "%a, %d %b %Y %H:%M:%S %Z")
            .map_err(|e| RaptoboError::new(&e.to_string()))?;

        let metadata = RepositoryMetadata {
            architectures: stanza_list("Architectures", &stanza)?,
            components: stanza_list("Components", &stanza)?,
            description: stanza_text("Description", &stanza)?,
            origin: stanza_opt_value("Origin", &stanza),
            label: stanza_opt_value("Label", &stanza),
            version: stanza_value("Version", &stanza)?,
            suite: stanza_opt_value("Suite", &stanza),
            codename: stanza_value("Codename", &stanza)?,
            date,
            md5sum: stanza_files("MD5Sum", &stanza)?,
            sha1: stanza_files("SHA1", &stanza)?,
            sha256: stanza_files("SHA256", &stanza)?,
        };

        Ok(metadata)
    }
}

#[derive(Debug)]
pub struct Repository {
    pub spec: RepositorySpec,
    pub metadata: Option<RepositoryMetadata>,
}

impl Repository {
    pub fn new(
        uri: &str,
        distribution: &str,
        components: Option<Vec<&str>>,
        source: bool,
        flat: bool,
    ) -> Repository {
        let c: Option<Vec<String>> = match components {
            None => None,
            Some(comps) => Some(comps.into_iter().map(|comp| comp.to_string()).collect()),
        };

        Repository {
            spec: RepositorySpec {
                flat,
                source,
                uri: uri.to_string(),
                distribution: distribution.to_string(),
                components: c,
            },
            metadata: None,
        }
    }

    fn inrelease_url(&self) -> String {
        if self.spec.flat {
            format!("{}/{}/InRelease", self.spec.uri, self.spec.distribution)
        } else {
            format!(
                "{}/dists/{}/InRelease",
                self.spec.uri, self.spec.distribution
            )
        }
    }

    pub fn load_metadata(&mut self) -> Result<(), RaptoboError> {
        let url = self.inrelease_url();

        log::debug!("[load_metadata] url: {}", url);

        let content = download(&url)?;

        let metadata = RepositoryMetadata::new(content)?;
        self.metadata = Some(metadata);

        Ok(())
    }
}
