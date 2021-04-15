use crate::parser;
use crate::{info, warn};
use anyhow::Result;
use console::style;
use git2::Repository;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    process::{Command, Stdio},
};

const AUR_RPC_URL: &str = "https://aur.archlinux.org/rpc/?v=5&type=info&arg[]=";
const AUR_URL: &str = "https://aur.archlinux.org/";

#[derive(Deserialize)]
struct Results {
    Version: String,
}

#[derive(Deserialize)]
struct APIResult {
    results: Vec<Results>,
}

fn fetch_version(pkgname: &str) -> Result<String> {
    let url = format!("{}{}", AUR_RPC_URL, pkgname.to_string());
    let resp = Client::new().get(&url).send()?;
    let apiresult: APIResult = resp.json()?;
    let results = &apiresult.results;
    let newver: &str;

    if results.len() == 0 {
        warn!("The package {} has been removed or cannot be accessed.", pkgname);
        newver = "null";
    } else {
        newver = &apiresult.results[0].Version;
    }

    Ok(newver.to_string())
}

fn fetch_update(pkgname: &str, pkgclone_path: &str) -> Result<()> {
    let pkgpath = format!("{}/{}", pkgclone_path, pkgname);
    let pkgurl = format!("{}{}.git", AUR_URL, pkgname);

    if parser::check_existance(&pkgpath) {
        info!("Updating the repo of {}...", pkgname);
        Command::new("git")
            .current_dir(Path::new(&pkgpath))
            .arg("pull")
            .stdout(Stdio::null())
            .spawn()?;
    } else {
        info!("Cloning the repo of {}...", pkgname);
        Repository::clone(&pkgurl, Path::new(&pkgpath))?;
    }

    Ok(())
}

pub fn fetch_updates(
    pkglist: &HashMap<String, String>,
    pkgclone_path: &str,
) -> Result<HashMap<String, String>> {
    let mut newpkglist: HashMap<String, String> = HashMap::new();
    let mut updatequeue: HashSet<String> = HashSet::new();

    for (pkgname, pkgver) in pkglist {
        info!("Fetching the version of {}...", &pkgname);
        let newver = fetch_version(&pkgname)?;
        if parser::strvercmp(&newver, &pkgver) {
            newpkglist.insert(pkgname.to_string(), newver);
            updatequeue.insert(pkgname.to_string());
        } else {
            newpkglist.insert(pkgname.to_string(), pkgver.to_string());
        }
    }

    for item in updatequeue.iter() {
        fetch_update(&item, pkgclone_path)?;
    }

    Ok(newpkglist)
}
