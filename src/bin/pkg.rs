#![deny(warnings)]

#[macro_use]
extern crate clap;
extern crate pkgutils;

use clap::{App, Arg, ArgMatches, SubCommand};
use pkgutils::commands::{run_clean_cmd, run_create_cmd, run_extract_cmd, run_fetch_cmd,
                         run_install_cmd, run_list_cmd, run_sign_cmd, run_upgrade_cmd};
use pkgutils::Repo;
use std::io::{self, Write};
use std::process;

fn run_pkg_cmd(args: &ArgMatches, repo: &Repo, fun: &Fn(&Repo, Vec<&str>) -> ()) {
    let packages: Vec<&str> = args.values_of("packages").unwrap().collect();
    if packages.len() == 0 {
        let _ = write!(io::stderr(), "pkg: no packages specified\n");
        process::exit(1);
    }
    fun(repo, packages);
}

fn main() {
    // let repo = Repo::new(env!("TARGET"));
    let repo = Repo::new(env!("PATH"));

    let args = App::new("pkg")
        .about("Pkg, the Redox package manager")
        .version(crate_version!())
        .subcommand(SubCommand::with_name("clean")
                        .about("clean an extracted package")
                        .arg(Arg::with_name("packages").help("The package to use").required(true).multiple(true)))
        .subcommand(SubCommand::with_name("create")
                        .about("create a package")
                        .arg(Arg::with_name("packages").help("The package to use").index(1).required(true)))
        .subcommand(SubCommand::with_name("extract")
                        .about("extract a package")
                        .arg(Arg::with_name("packages").help("The package to use").index(1).required(true)))
        .subcommand(SubCommand::with_name("fetch")
                        .about("download a package")
                        .arg(Arg::with_name("packages").help("The package to use").index(1).required(true)))
        // TODO: set new commands
        .subcommand(SubCommand::with_name("install")
                        .about("install a package")
                        .arg(Arg::with_name("packages").help("The package to use").index(1).required(true)))
        .subcommand(SubCommand::with_name("list")
                        .about("list package contents")
                        .arg(Arg::with_name("packages").help("The package to use").index(1).required(true)))
        .subcommand(SubCommand::with_name("sign")
                        .about("get a file signature")
                        .arg(Arg::with_name("files").help("The files to sign").index(1).required(true)))
        .subcommand(SubCommand::with_name("upgrade")
                        .about("upgrade all package"))
        .arg(Arg::with_name("verbose")
                 .short("v")
                 .multiple(true)
                 .help("verbosity level"))
        .get_matches();

    match args.subcommand() {
        ("clean", Some(c)) => {
            run_pkg_cmd(c, &repo, &run_clean_cmd);
        }
        ("create", Some(c)) => {
            run_pkg_cmd(c, &repo, &run_create_cmd);
        }
        ("extract", Some(c)) => {
            run_pkg_cmd(c, &repo, &run_extract_cmd);
        }
        ("fetch", Some(c)) => {
            run_pkg_cmd(c, &repo, &run_fetch_cmd);
        }
        ("install", Some(c)) => {
            run_pkg_cmd(c, &repo, &run_install_cmd);
        }
        ("list", Some(c)) => {
            run_pkg_cmd(c, &repo, &run_list_cmd);
        }
        ("sign", Some(c)) => {
            run_pkg_cmd(c, &repo, &run_sign_cmd);
        }
        ("upgrade", _) => {
            run_upgrade_cmd(repo);
        }
        _ => {
            let _ = write!(io::stderr(), "Command failed!\n");
            process::exit(1);
        }
    }
}
