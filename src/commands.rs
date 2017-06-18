use liner;
use {Repo, Package, PackageMeta, PackageMetaList};
use std::{env};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use version_compare::{VersionCompare, CompOp};

fn upgrade(repo: Repo) -> io::Result<()> {
    let mut local_list = PackageMetaList::new();
    if Path::new("/pkg/").is_dir() {
        for entry_res in fs::read_dir("/pkg/")? {
            let entry = entry_res?;

            let mut toml = String::new();
            File::open(entry.path())?.read_to_string(&mut toml)?;

            if let Ok(package) = PackageMeta::from_toml(&toml) {
                local_list.packages.insert(package.name, package.version);
            }
        }
    }

    let tomlfile = repo.sync("repo.toml")?;

    let mut toml = String::new();
    File::open(tomlfile)?.read_to_string(&mut toml)?;

    let remote_list =
        PackageMetaList::from_toml(&toml)
            .map_err(|err| {
                         io::Error::new(io::ErrorKind::InvalidData, format!("TOML error: {}", err))
                     })?;

    let mut upgrades = Vec::new();
    for (package, version) in local_list.packages.iter() {
        let remote_version = remote_list.packages.get(package).map_or("", |s| &s);
        match VersionCompare::compare(version, remote_version) {
            Ok(cmp) => {
                match cmp {
                    CompOp::Lt => {
                        upgrades.push((package.clone(),
                                       version.clone(),
                                       remote_version.to_string()));
                    }
                    _ => (),
                }
            }
            Err(_err) => {
                println!("{}: version parsing error when comparing {} and {}",
                         package,
                         version,
                         remote_version);
            }
        }
    }

    if upgrades.is_empty() {
        println!("All packages are up to date.");
    } else {
        for &(ref package, ref old_version, ref new_version) in upgrades.iter() {
            println!("{}: {} => {}", package, old_version, new_version);
        }

        let line = liner::Context::new()
            .read_line("Do you want to upgrade these packages? (Y/n) ", &mut |_| {})?;
        match line.to_lowercase().as_str() {
            "" | "y" | "yes" => {
                println!("Downloading packages");
                let mut packages = Vec::new();
                for (package, _, _) in upgrades {
                    packages.push(repo.fetch(&package)?);
                }

                println!("Installing packages");
                for mut package in packages {
                    package.install("/")?;
                }
            }
            _ => {
                println!("Cancelling upgrade.");
            }
        }
    }

    Ok(())
}

pub fn run_clean_cmd(repo: &Repo, packages: Vec<&str>) {
    for package in packages.iter() {
        match repo.clean(package) {
            Ok(tardir) => {
                let _ = write!(io::stderr(),
                               "pkg: clean: {}: cleaned {}\n",
                               package,
                               tardir);
            }
            Err(err) => {
                let _ = write!(io::stderr(), "pkg: clean: {}: failed: {}\n", package, err);
            }
        }
    }
}

pub fn run_create_cmd(repo: &Repo, packages: Vec<&str>) {
    for package in packages.iter() {
        match repo.create(package) {
            Ok(tarfile) => {
                let _ = write!(io::stderr(),
                               "pkg: create: {}: created {}\n",
                               package,
                               tarfile);
            }
            Err(err) => {
                let _ = write!(io::stderr(), "pkg: create: {}: failed: {}\n", package, err);
            }
        }
    }
}

pub fn run_extract_cmd(repo: &Repo, packages: Vec<&str>) {
    for package in packages.iter() {
        match repo.extract(package) {
            Ok(tardir) => {
                let _ = write!(io::stderr(),
                               "pkg: extract: {}: extracted to {}\n",
                               package,
                               tardir);
            }
            Err(err) => {
                let _ = write!(io::stderr(), "pkg: extract: {}: failed: {}\n", package, err);
            }
        }
    }
}

pub fn run_fetch_cmd(repo: &Repo, packages: Vec<&str>) {
    for package in packages.iter() {
        match repo.fetch(package) {
            Ok(pkg) => {
                let _ = write!(io::stderr(),
                               "pkg: fetch: {}: fetched {}\n",
                               package,
                               pkg.path().display());
            }
            Err(err) => {
                let _ = write!(io::stderr(), "pkg: fetch: {}: failed: {}\n", package, err);
            }
        }
    }
}

pub fn run_install_cmd(repo: &Repo, packages: Vec<&str>) {
    for package in packages.iter() {
        let pkg = if package.ends_with(".tar") {
            let path = format!("{}/{}",
                               env::current_dir().unwrap().to_string_lossy(),
                               package);
            Package::from_path(&path)
        } else {
            repo.fetch(package)
        };

        if let Err(err) = pkg.and_then(|mut p| p.install("/")) {
            let _ = write!(io::stderr(), "pkg: install: {}: failed: {}\n", package, err);
        } else {
            let _ = write!(io::stderr(), "pkg: install: {}: succeeded\n", package);
        }
    }
}

pub fn run_list_cmd(repo: &Repo, packages: Vec<&str>) {
    for package in packages.iter() {
        if let Err(err) = repo.fetch(package).and_then(|mut p| p.list()) {
            let _ = write!(io::stderr(), "pkg: list: {}: failed: {}\n", package, err);
        } else {
            let _ = write!(io::stderr(), "pkg: list: {}: succeeded\n", package);
        }
    }
}

pub fn run_sign_cmd(repo: &Repo, files: Vec<&str>) {
    for file in files.iter() {
        match repo.signature(file) {
            Ok(signature) => {
                let _ = write!(io::stderr(), "pkg: sign: {}: {}\n", file, signature);
            }
            Err(err) => {
                let _ = write!(io::stderr(), "pkg: sign: {}: failed: {}\n", file, err);
            }
        }
    }
}

pub fn run_upgrade_cmd(repo: Repo) {
    match upgrade(repo) {
                Ok(()) => {
                        let _ = write!(io::stderr(), "pkg: upgrade: succeeded\n");
                    }
                Err(err) => {
                    let _ = write!(io::stderr(), "pkg: upgrade: failed: {}\n", err);
                }
            }
}

// run_upgrade_cmd