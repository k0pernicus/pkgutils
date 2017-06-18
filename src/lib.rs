#![deny(warnings)]

extern crate hyper;
extern crate hyper_rustls;
extern crate liner;
extern crate octavo;
#[macro_use]
extern crate serde_derive;
extern crate tar;
extern crate toml;
extern crate version_compare;

use octavo::octavo_digest::Digest;
use octavo::octavo_digest::sha3::Sha512;
use std::str;
use std::fs::{self, File};
use std::io::{self, stderr, Read, Write};
use std::path::Path;

pub use download::download;
pub use packagemeta::{PackageMeta, PackageMetaList};
pub use package::Package;

pub mod commands;
mod download;
mod packagemeta;
mod package;

pub struct Repo {
    local: String,
    remotes: Vec<String>,
    target: String,
}

impl Repo {
    pub fn new(target: &str) -> Repo {
        let mut remotes = vec![];

        //TODO: Cleanup
        // This will add every line in every file in /etc/pkg.d to the remotes,
        // provided it does not start with #
        {
            let mut entries = vec![];
            if let Ok(read_dir) = fs::read_dir("/etc/pkg.d") {
                for entry_res in read_dir {
                    if let Ok(entry) = entry_res {
                        let path = entry.path();
                        if path.is_file() {
                            entries.push(path);
                        }
                    }
                }
            }

            entries.sort();

            for entry in entries {
                if let Ok(mut file) = File::open(entry) {
                    let mut data = String::new();
                    if let Ok(_) = file.read_to_string(&mut data) {
                        for line in data.lines() {
                            if !line.starts_with('#') {
                                remotes.push(line.to_string());
                            }
                        }
                    }
                }
            }
        }

        Repo {
            local: format!("/tmp/pkg"),
            remotes: remotes,
            target: target.to_string(),
        }
    }

    pub fn sync(&self, file: &str) -> io::Result<String> {
        let local_path = format!("{}/{}", self.local, file);

        if let Some(parent) = Path::new(&local_path).parent() {
            fs::create_dir_all(parent)?;
        }

        let mut res = Err(io::Error::new(io::ErrorKind::NotFound, format!("no remote paths")));
        for remote in self.remotes.iter() {
            let remote_path = format!("{}/{}/{}", remote, self.target, file);
            res = download(&remote_path, &local_path).map(|_| local_path.clone());
            if res.is_ok() {
                break;
            }
        }
        res
    }

    pub fn signature(&self, file: &str) -> io::Result<String> {
        let mut data = vec![];
        File::open(&file)?.read_to_end(&mut data)?;

        let mut output = vec![0; Sha512::output_bytes()];
        let mut hash = Sha512::default();
        hash.update(&data);
        hash.result(&mut output);

        let mut encoded = String::new();
        for b in output.iter() {
            encoded.push_str(&format!("{:X}", b));
        }

        Ok(encoded)
    }

    pub fn clean(&self, package: &str) -> io::Result<String> {
        let tardir = format!("{}/{}", self.local, package);
        fs::remove_dir_all(&tardir)?;
        Ok(tardir)
    }

    pub fn create(&self, package: &str) -> io::Result<String> {
        if !Path::new(package).is_dir() {
            return Err(io::Error::new(io::ErrorKind::NotFound, format!("{} not found", package)));
        }

        let sigfile = format!("{}.sig", package);
        let tarfile = format!("{}.tar", package);

        {
            let file = File::create(&tarfile)?;
            let mut tar = tar::Builder::new(file);
            tar.append_dir_all("", package)?;
            tar.finish()?;
        }

        let mut signature = self.signature(&tarfile)?;
        signature.push('\n');

        File::create(&sigfile)?
            .write_all(&signature.as_bytes())?;

        Ok(tarfile)
    }

    pub fn fetch(&self, package: &str) -> io::Result<Package> {
        let sigfile = self.sync(&format!("{}.sig", package))?;

        let mut expected = String::new();
        File::open(sigfile)?.read_to_string(&mut expected)?;
        let expected = expected.trim();

        {
            let tarfile = format!("{}/{}.tar", self.local, package);
            if let Ok(signature) = self.signature(&tarfile) {
                if signature == expected {
                    write!(stderr(), "* Already downloaded {}\n", package)?;
                    return Package::from_path(tarfile);
                }
            }
        }

        let tarfile = self.sync(&format!("{}.tar", package))?;

        if self.signature(&tarfile)? != expected {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                                      format!("{} not valid", package)));
        }

        Package::from_path(tarfile)
    }

    pub fn extract(&self, package: &str) -> io::Result<String> {
        let tardir = format!("{}/{}", self.local, package);
        fs::create_dir_all(&tardir)?;
        self.fetch(package)?.install(&tardir)?;
        Ok(tardir)
    }

    pub fn add_remote(&mut self, remote: &str) {
        self.remotes.push(remote.to_string());
    }
}
