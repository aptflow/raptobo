use crate::error::RaptoboError;
use crate::utils::{
    stanza_date, stanza_lines, stanza_opt_files, stanza_opt_list, stanza_opt_text,
    stanza_opt_value, stanza_value, File, parse_metadata
};
use chrono::{DateTime, FixedOffset};
use std::cmp::{max, Ordering};
use std::collections::HashMap;
use std::iter::repeat;

#[derive(Debug)]
pub struct PackageMetadata {
    // Debian control values
    /// Source package name, Debian policy 5.6.1
    pub source: Option<String>,
    /// Maintainers name and email address, RFC822 format, Debian policy 5.6.2
    pub maintainer: Option<String>,
    /// List of the names and email addresses of co-maintainers of the package, Debian policy 5.6.3
    pub uploaders: Option<Vec<String>>,
    /// Name and email address of the person who prepared this version, RFC822 format, Debian policy 5.6.2
    pub changed_by: Option<String>,
    /// Section, Debian policy 5.6.5, allowed values see Debian policy 2.4
    pub section: Option<String>,
    /// Section, Debian policy 5.6.6, allowed values see Debian policy 2.5
    pub priority: Option<String>,
    /// Name, Debian policy 5.6.7
    pub package: String,
    /// Architecture, Debian policy 5.6.8
    pub architecture: String,
    /// Essential, cannot be removed, boolean field, values: yes or no, Debian policy 5.6.9
    pub essential: Option<String>,
    // Package relationships, Debian Policy 7
    /// declares an absolute dependency
    pub depends: Option<Vec<PackageRelation>>,
    /// Pre-Depends is like Depends, but forces that the installation of the linked packages is completed
    pub pre_depends: Option<Vec<PackageRelation>>,
    /// strong, but not absolute, dependency
    pub recommends: Option<Vec<PackageRelation>>,
    /// may be more useful with linked ones
    pub suggests: Option<Vec<PackageRelation>>,
    /// like suggests, but other direction
    pub enhances: Option<Vec<PackageRelation>>,
    /// breaks linked packages
    pub breaks: Option<Vec<PackageRelation>>,
    /// conflicts with linked packages
    pub conflicts: Option<Vec<PackageRelation>>,
    /// provides the named virtual packages, see Debian Policy 7.5
    pub provides: Option<Vec<PackageRelation>>,
    /// overwrites files of linked packages, see Debian Policy 7.6
    pub replaces: Option<Vec<PackageRelation>>,
    /// Version, Debian policy 5.6.11
    pub standards_version: Option<String>,
    /// Version, Debian policy 5.6.12
    pub version: PackageVersion,
    /// Package description, Debian policy 5.6.13
    pub description: Option<String>,
    /// List of distribution names containing this package, Debian Policy 5.6.14
    pub distribution: Option<Vec<String>>,
    /// Date when the package was last built, Debian Policy 5.6.15
    pub date: Option<DateTime<FixedOffset>>,
    /// Package format, Debian Policy 5.6.16
    pub format: Option<String>,
    /// Urgency, how important is it to install the new version, Debian Policy 5.6.17
    pub urgency: Option<PackageUpdateUrgency>,
    /// human-readable changes description, Debian Policy 5.6.18
    pub changes: Option<String>,
    /// list of binary package names built from this source, Debian Policy 5.6.19
    pub binary: Option<Vec<String>>,
    /// estimated installed package size in bytes, Debian Policy 5.6.20
    pub installed_size: Option<String>,
    /// list of files which are part of this source package with md5 checksums, Debian Policy 5.6.21
    pub files: Option<Vec<File>>,
    /// list of bugs closed with this version, Debian Policy 5.6.22
    pub closes: Option<Vec<String>>,
    /// Homepage of the project this package belongs to, Debian Policy 5.6.23
    pub homepage: Option<String>,
    /// list of files which are part of this source package with md5 checksums, Debian Policy 5.6.24
    pub checksums_sha1: Option<Vec<File>>,
    /// list of files which are part of this source package with md5 checksums, Debian Policy 5.6.24
    pub checksums_sha256: Option<Vec<File>>,
    /// web interface for browsing the source repository, Debian Policy 5.6.26
    pub vcs_browser: Option<String>,
    /// source repository, Debian Policy 5.6.26
    pub vcs: Option<PackageVcs>,
    /// list of packages built by this source package, Debian Policy 5.6.27
    pub package_list: Option<Vec<PackageListItem>>,
    /// package type, deb or udeb, Debian Policy 5.6.28
    pub package_type: Option<String>,
    /// git hash of package, Debian Policy 5.6.29
    pub dgit: Option<String>,
    /// list of test names, Debian Policy 5.6.30
    pub testsuite: Option<Vec<String>>,
    /// root or fakeroot is needed to build the package, Debian Policy 5.6.31
    pub rules_requires_root: Option<String>,
    /// origin of the package
    pub origin: Option<String>,
    /// maintainer of the original package
    pub original_maintainer: Option<String>,
    /// link to the bug-tracker
    pub bugs: Option<String>,
    /// list of tasks
    pub task: Option<Vec<String>>,
    // Package index additional values
    /// path to file, relative to the base of the repository
    pub filename: Option<String>,
    /// compressed size, as bytes
    pub size: Option<String>,
    /// md5 hash of the package binary package
    pub md5sum: Option<String>,
    /// sha1 hash of the package binary package
    pub sha1: Option<String>,
    /// sha256 hash of the package binary package
    pub sha256: Option<String>,
    /// sha512 hash of the package binary package
    pub sha512: Option<String>,
    /// lookup key for translations
    pub description_md5: Option<String>,
}

impl PackageMetadata {
    pub fn new(stanza: HashMap<String, Vec<String>>) -> Result<PackageMetadata, RaptoboError> {
        Ok(PackageMetadata {
            source: stanza_opt_value("Source", &stanza),
            maintainer: stanza_opt_value("Maintainer", &stanza),
            uploaders: stanza_opt_list("Uploaders", &stanza),
            changed_by: stanza_opt_value("Changed-By", &stanza),
            section: stanza_opt_value("Section", &stanza),
            priority: stanza_opt_value("Priority", &stanza),
            package: stanza_value("Package", &stanza)?,
            architecture: stanza_value("Architecture", &stanza)?,
            essential: stanza_opt_value("Essential", &stanza),
            depends: PackageRelation::parse("Depends", &stanza),
            pre_depends: PackageRelation::parse("Pre-Depends", &stanza),
            recommends: PackageRelation::parse("Recommends", &stanza),
            suggests: PackageRelation::parse("Suggests", &stanza),
            enhances: PackageRelation::parse("Enhances", &stanza),
            breaks: PackageRelation::parse("Breaks", &stanza),
            conflicts: PackageRelation::parse("Conflicts", &stanza),
            provides: PackageRelation::parse("Provides", &stanza),
            replaces: PackageRelation::parse("Replaces", &stanza),
            standards_version: stanza_opt_value("Standards-Version", &stanza),
            version: PackageVersion::parse("Version", &stanza)?,
            description: stanza_opt_text("Description", &stanza),
            distribution: stanza_opt_list("Distribution", &stanza),
            date: stanza_date("Date", &stanza),
            format: stanza_opt_value("Format", &stanza),
            urgency: PackageUpdateUrgency::parse("Urgency", &stanza),
            changes: stanza_opt_value("Changes", &stanza),
            binary: stanza_opt_list("Binary", &stanza),
            installed_size: stanza_opt_value("Installed-Size", &stanza),
            files: stanza_opt_files("Files", &stanza),
            closes: stanza_opt_list("Closes", &stanza),
            homepage: stanza_opt_value("Homepage", &stanza),
            checksums_sha1: stanza_opt_files("Checksums-Sha1", &stanza),
            checksums_sha256: stanza_opt_files("Checksums-Sha256", &stanza),
            vcs_browser: stanza_opt_value("Vcs-Browser", &stanza),
            vcs: PackageVcs::parse(&stanza),
            package_list: PackageListItem::parse("Package-List", &stanza),
            package_type: stanza_opt_value("Package-Type", &stanza),
            dgit: stanza_opt_value("Dgit", &stanza),
            testsuite: stanza_opt_list("Testsuite", &stanza),
            rules_requires_root: stanza_opt_value("Rules-Requires-Root", &stanza),
            origin: stanza_opt_value("Origin", &stanza),
            original_maintainer: stanza_opt_value("Original-Maintainer", &stanza),
            bugs: stanza_opt_value("Bugs", &stanza),
            task: stanza_opt_list("Task", &stanza),
            filename: stanza_opt_value("Filename", &stanza),
            size: stanza_opt_value("Size", &stanza),
            md5sum: stanza_opt_value("MD5sum", &stanza),
            sha1: stanza_opt_value("SHA1", &stanza),
            sha256: stanza_opt_value("SHA256", &stanza),
            sha512: stanza_opt_value("SHA512", &stanza),
            description_md5: stanza_opt_value("Description-md5", &stanza),
        })
    }

    pub fn parse(content: Vec<String>) -> Result<Vec<PackageMetadata>, RaptoboError> {
        let stanzas = parse_metadata(content)?;

        Ok(stanzas.into_iter()
        .map(|s| PackageMetadata::new(s))
        .filter(|r| match r {
            Ok(_) => true,
            Err(e) => {
                log::error!("[PackageMetadata::parse] error: {}", e);
                false
            }
        })
        .map(|r| r.unwrap())
        .collect())
    }
}

#[derive(Debug, Clone)]
pub struct PackageListItem {
    pub name: String,
    pub type_name: String,
    pub section: String,
    pub priority: String,
}

impl PackageListItem {
    pub fn parse(key: &str, stanza: &HashMap<String, Vec<String>>) -> Option<Vec<PackageListItem>> {
        let lines = match stanza_lines(key, &stanza, true) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let items: Vec<PackageListItem> = lines
            .into_iter()
            .map(|l| {
                let parts: Vec<&str> = l.split(" ").collect();
                PackageListItem {
                    name: parts[0].trim().to_string(),
                    type_name: parts[1].trim().to_string(),
                    section: parts[3].trim().to_string(),
                    priority: parts[4].trim().to_string(),
                }
            })
            .collect();

        if items.is_empty() {
            None
        } else {
            Some(items)
        }
    }
}

#[derive(Debug, Clone)]
pub enum VcsType {
    Arch,
    Bzr,
    Cvs,
    Darcs,
    Git,
    Hg,
    Mtn,
    Svn,
}

#[derive(Debug, Clone)]
pub struct PackageVcs {
    pub vcs_type: VcsType,
    pub url: String,
}

impl PackageVcs {
    pub fn parse(stanza: &HashMap<String, Vec<String>>) -> Option<PackageVcs> {
        let types = vec![
            (VcsType::Arch, "Vcs-Arch"),
            (VcsType::Bzr, "Vcs-Bzr"),
            (VcsType::Cvs, "Vcs-Cvs"),
            (VcsType::Darcs, "Vcs-Darcs"),
            (VcsType::Git, "Vcs-Git"),
            (VcsType::Hg, "Vcs-Hg"),
            (VcsType::Mtn, "Vcs-Mtn"),
            (VcsType::Svn, "Vcs-Svn"),
        ];

        for (t, key) in types.into_iter() {
            match stanza_value(key, &stanza) {
                Ok(v) => {
                    return Some(PackageVcs {
                        vcs_type: t,
                        url: v,
                    })
                }
                Err(_) => continue,
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub enum PackageUpdateUrgency {
    Low,
    Medium,
    High,
    Emergency,
    Critical,
}

impl PackageUpdateUrgency {
    pub fn parse(key: &str, stanza: &HashMap<String, Vec<String>>) -> Option<PackageUpdateUrgency> {
        let value = match stanza_value(key, &stanza) {
            Ok(v) => v,
            Err(_) => return None,
        };

        if value.trim().to_lowercase() == "low" {
            Some(PackageUpdateUrgency::Low)
        } else if value.trim().to_lowercase() == "medium" {
            Some(PackageUpdateUrgency::Medium)
        } else if value.trim().to_lowercase() == "high" {
            Some(PackageUpdateUrgency::High)
        } else if value.trim().to_lowercase() == "emergency" {
            Some(PackageUpdateUrgency::Emergency)
        } else if value.trim().to_lowercase() == "critical" {
            Some(PackageUpdateUrgency::Critical)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum PackageVersionRelation {
    LT,
    LTE,
    EQ,
    GTE,
    GT,
}

impl PackageVersionRelation {
    pub fn new(rel: &str) -> Result<PackageVersionRelation, RaptoboError> {
        if rel == "<<" {
            return Ok(PackageVersionRelation::LT);
        } else if rel == "<=" {
            return Ok(PackageVersionRelation::LTE);
        } else if rel == "=" {
            return Ok(PackageVersionRelation::EQ);
        } else if rel == ">=" {
            return Ok(PackageVersionRelation::GTE);
        } else if rel == ">>" {
            return Ok(PackageVersionRelation::GT);
        } else {
            return Err(RaptoboError::new(&format!(
                "[PackageVersionRelation] unknown relation {}",
                rel
            )));
        }
    }

    pub fn is(&self, o: Ordering) -> bool {
        match o {
            Ordering::Equal => match self {
                PackageVersionRelation::LTE
                | PackageVersionRelation::EQ
                | PackageVersionRelation::GTE => true,
                _ => false,
            },
            Ordering::Less => match self {
                PackageVersionRelation::LTE | PackageVersionRelation::LT => true,
                _ => false,
            },
            Ordering::Greater => match self {
                PackageVersionRelation::GT | PackageVersionRelation::GTE => true,
                _ => false,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackageRelation {
    pub package: String,
    pub relation: PackageVersionRelation,
    pub version: Option<PackageVersion>,
    pub alternative: Option<Box<PackageRelation>>,
}

impl PackageRelation {
    pub fn parse(key: &str, stanza: &HashMap<String, Vec<String>>) -> Option<Vec<PackageRelation>> {
        let value = match stanza_value(key, &stanza) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let list: Vec<PackageRelation> = value
            .split(",")
            .into_iter()
            .map(|r| r.trim())
            .map(|r| PackageRelation::new(r))
            .filter(|r| match r {
                Ok(_) => false,
                Err(e) => {
                    log::error!("[PackageRelation::parse] relation parse error: {}", e);
                    true
                }
            })
            .map(|r| r.unwrap())
            .collect();

        if list.is_empty() {
            None
        } else {
            Some(list)
        }
    }

    pub fn new(relation: &str) -> Result<PackageRelation, RaptoboError> {
        // see Debian Policy 7.1
        let relation = relation.trim();
        let (r, a) = match relation.split_once("|") {
            Some((r, a)) => (r.trim(), Some(Box::new(PackageRelation::new(a.trim())?))),
            None => (relation, None),
        };

        let (name, version) = match r.split_once(" ") {
            None => {
                return Ok(PackageRelation {
                    package: relation.to_string(),
                    relation: PackageVersionRelation::EQ,
                    version: None,
                    alternative: a,
                })
            }
            Some((name, version)) => (name.trim(), version.trim()),
        };
        let version = &version[1..version.len() - 1];
        let (rel, ver) = match version.split_once(" ") {
            None => {
                return Err(RaptoboError::new(&format!(
                    "[PackageRelation] invalid version {}",
                    version
                )))
            }
            Some((rel, ver)) => (rel.trim(), ver.trim()),
        };
        let relation = PackageVersionRelation::new(rel)?;
        let version = PackageVersion::new(ver)?;

        Ok(PackageRelation {
            package: name.to_string(),
            relation,
            version: Some(version),
            alternative: a,
        })
    }

    pub fn is(&self, package: &PackageMetadata) -> bool {
        let mut p = self;

        loop {
            if p.package == package.package {
                match &p.version {
                    None => {
                        return true;
                    }
                    Some(v) => match v.partial_cmp(&package.version) {
                        None => {
                            return false;
                        }
                        Some(ord) => {
                            return p.relation.is(ord);
                        }
                    },
                }
            } else if let Some(alternative) = &p.alternative {
                p = alternative;
            } else {
                return false;
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PackageVersion {
    pub epoch: u64,
    pub upstream_version: Version,
    pub debian_revision: Version,
}

impl PackageVersion {
    pub fn parse(
        key: &str,
        stanza: &HashMap<String, Vec<String>>,
    ) -> Result<PackageVersion, RaptoboError> {
        let value = stanza_value(key, stanza)?;
        PackageVersion::new(&value)
    }

    pub fn new(version: &str) -> Result<PackageVersion, RaptoboError> {
        let res = version.split_once(":");
        let (epoch, tail) = match res {
            Some((e, r)) => {
                let epoch = e
                    .parse::<u64>()
                    .map_err(|err| RaptoboError::new(&err.to_string()))?;
                (epoch, r)
            }
            None => (0, version),
        };

        let res = tail.split_once("-");
        let (upstream_version, debian_revision) = match res {
            Some((v, r)) => (v, r),
            None => (tail, ""),
        };

        Ok(PackageVersion {
            epoch,
            upstream_version: Version::new(upstream_version),
            debian_revision: Version::new(debian_revision),
        })
    }
}

impl PartialEq<str> for PackageVersion {
    fn eq(&self, version: &str) -> bool {
        let (epoch, tail) = match version.split_once(":") {
            Some((e, r)) => {
                let epoch: u64 = match e.parse::<u64>() {
                    Ok(e) => e,
                    Err(_) => 0,
                };
                (epoch, r)
            }
            None => (0, version),
        };

        let res = tail.split_once("-");
        let (upstream_version, debian_revision) = match res {
            Some((v, r)) => (v, Some(r)),
            None => (tail, None),
        };

        let eq = self.epoch == epoch && self.upstream_version.version == upstream_version;
        let eq = match debian_revision {
            Some(r) => eq && self.debian_revision.version == r,
            None => eq,
        };

        eq
    }
}

impl PartialOrd for PackageVersion {
    fn partial_cmp(&self, other: &PackageVersion) -> Option<Ordering> {
        if self.epoch != other.epoch {
            self.epoch.partial_cmp(&other.epoch)
        } else if self.upstream_version != other.upstream_version {
            self.upstream_version.partial_cmp(&other.upstream_version)
        } else {
            self.debian_revision.partial_cmp(&other.debian_revision)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct VersionBlock {
    pub prefix: String,
    pub number: u64,
}

impl VersionBlock {
    fn new() -> VersionBlock {
        VersionBlock {
            prefix: String::new(),
            number: 0,
        }
    }

    fn from(version: &str) -> Vec<VersionBlock> {
        if version.len() == 0 {
            return Vec::new();
        }

        let mut blocks: Vec<VersionBlock> = Vec::new();

        let mut start = 0;
        let mut start_digit = 0;
        let mut digit = false;

        for (i, c) in version.chars().enumerate() {
            if c.is_ascii_digit() {
                digit = true;
                start_digit = i;
                continue;
            }

            if !c.is_ascii_digit() && digit {
                let prefix = version[start..start_digit].to_string();
                let number = match version[start_digit..i].parse::<u64>() {
                    Ok(n) => n,
                    Err(e) => {
                        log::error!(
                            "[VersionBlock::from] invalid number: {} - {}",
                            &version[start_digit..i],
                            e
                        );
                        0
                    }
                };
                blocks.push(VersionBlock { prefix, number });

                digit = false;
                start = i;
            }
        }

        let len = version.len();
        if start_digit < start {
            start_digit = len;
        }
        let prefix = version[start..start_digit].to_string();
        let number = match version[start_digit..len].parse::<u64>() {
            Ok(n) => n,
            Err(e) => {
                log::error!(
                    "[VersionBlock::from] invalid number: {} - {}",
                    &version[start_digit..len],
                    e
                );
                0
            }
        };
        blocks.push(VersionBlock { prefix, number });

        blocks
    }
}

impl PartialOrd for VersionBlock {
    fn partial_cmp(&self, other: &VersionBlock) -> Option<Ordering> {
        if (self.prefix.is_empty() && other.prefix.is_empty()) || (self.prefix == other.prefix) {
            return self.number.partial_cmp(&other.number);
        }

        if self.prefix.is_empty() {
            if other.prefix.chars().next().unwrap() == '~' {
                return Some(Ordering::Greater);
            } else {
                return Some(Ordering::Less);
            }
        }

        if other.prefix.is_empty() {
            if self.prefix.chars().next().unwrap() == '~' {
                return Some(Ordering::Less);
            } else {
                return Some(Ordering::Greater);
            }
        }

        for (s, o) in self.prefix.chars().zip(other.prefix.chars()) {
            if s != o {
                if s == '~' {
                    return Some(Ordering::Less);
                } else if o == '~' {
                    return Some(Ordering::Greater);
                } else {
                    return s.partial_cmp(&o);
                }
            }
        }

        self.prefix.len().partial_cmp(&other.prefix.len())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Version {
    pub version: String,
}

impl Version {
    pub fn new(version: &str) -> Version {
        Version {
            version: version.to_string(),
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        let sl = VersionBlock::from(&self.version);
        let ol = VersionBlock::from(&other.version);

        let len = max(sl.len(), ol.len());

        let sl = sl.into_iter().chain(repeat(VersionBlock::new())).take(len);
        let ol = ol.into_iter().chain(repeat(VersionBlock::new())).take(len);

        for (sb, ob) in sl.zip(ol) {
            match sb.partial_cmp(&ob) {
                None => panic!("[Version::partial_cmp] blocks not compareable!"),
                Some(o) => match o {
                    Ordering::Equal => continue,
                    Ordering::Greater => return Some(Ordering::Greater),
                    Ordering::Less => return Some(Ordering::Less),
                },
            }
        }

        Some(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::{PackageVersion, Version, VersionBlock};

    #[test]
    fn version_parsing_works() {
        let v = "1.2.6-1ubuntu1";
        let v = PackageVersion::new(v).unwrap();

        assert_eq!(v.epoch, 0);
        assert_eq!(v.upstream_version, Version::new("1.2.6"));
        assert_eq!(v.debian_revision, Version::new("1ubuntu1"));

        let v = "3.20191218.1ubuntu2";
        let v = PackageVersion::new(v).unwrap();
        assert_eq!(v.epoch, 0);
        assert_eq!(v.upstream_version, Version::new("3.20191218.1ubuntu2"));
        assert_eq!(v.debian_revision, Version::new(""));

        let v = "1.2.3-4.5.6";
        let v = PackageVersion::new(v).unwrap();
        assert_eq!(v.epoch, 0);
        assert_eq!(v.upstream_version, Version::new("1.2.3"));
        assert_eq!(v.debian_revision, Version::new("4.5.6"));

        let v = "1:1.2.3-4.5.6";
        let v = PackageVersion::new(v).unwrap();
        assert_eq!(v.epoch, 1);
        assert_eq!(v.upstream_version, Version::new("1.2.3"));
        assert_eq!(v.debian_revision, Version::new("4.5.6"));
    }

    #[test]
    fn compare_versions_epoch() {
        let v1 = PackageVersion::new("1.2.3-4.5.6").unwrap();
        let v2 = PackageVersion::new("1:1.2.3-4.5.6").unwrap();

        assert!(v1 < v2);
        assert!(v1 == v1.clone())
    }

    #[test]
    fn compare_versions_upstream() {
        let v1 = PackageVersion::new("1.2.3-4.5.6").unwrap();
        let v2 = PackageVersion::new("1.2.4-4.5.6").unwrap();

        assert!(v1 < v2);
    }

    #[test]
    fn compare_versions_upstream_tilde() {
        let v1 = PackageVersion::new("1.2.3-4.5.6").unwrap();
        let v2 = PackageVersion::new("~1-4.5.6").unwrap();

        assert!(v2 < v1);
    }

    #[test]
    fn compare_versions_debian() {
        let v1 = PackageVersion::new("1.2.3-4.5.6").unwrap();
        let v2 = PackageVersion::new("1.2.3-4.6.6").unwrap();

        assert!(v1 < v2);
    }

    #[test]
    fn compare_versions_debian_tilde() {
        let v1 = PackageVersion::new("1.2.3-4.5.6").unwrap();
        let v2 = PackageVersion::new("1.2.3-~6").unwrap();

        assert!(v2 < v1);
    }

    #[test]
    fn compare_versions() {
        let v1 = VersionBlock {
            prefix: String::from(""),
            number: 1,
        };
        let v2 = VersionBlock {
            prefix: String::from(""),
            number: 2,
        };
        assert!(v1 < v2);

        let v1 = VersionBlock {
            prefix: String::from(""),
            number: 1,
        };
        let v2 = VersionBlock {
            prefix: String::from(""),
            number: 1,
        };
        assert!(v1 == v2);

        let v1 = VersionBlock {
            prefix: String::from("b"),
            number: 1,
        };
        let v2 = VersionBlock {
            prefix: String::from("a"),
            number: 2,
        };
        assert!(v2 < v1);

        let v1 = VersionBlock {
            prefix: String::from(""),
            number: 1,
        };
        let v2 = VersionBlock {
            prefix: String::from("~"),
            number: 2,
        };
        assert!(v2 < v1);
    }

    #[test]
    fn version_blocks() {
        let blocks = VersionBlock::from("1.2.3");

        assert_eq!(blocks.len(), 3);

        assert_eq!(blocks[0].number, 1);
        assert_eq!(blocks[0].prefix, "");

        assert_eq!(blocks[1].number, 2);
        assert_eq!(blocks[1].prefix, ".");

        assert_eq!(blocks[2].number, 3);
        assert_eq!(blocks[2].prefix, ".");
    }
}
