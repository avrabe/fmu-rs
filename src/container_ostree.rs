use crate::ostree::OstreeOpts;
use crate::utils::path_is_empty;
use ostree::gio::Cancellable;
use ostree::glib::prelude::*; // or `use gtk::prelude::*;`
use ostree::glib::VariantDict;
use ostree::{
    AsyncProgress, RepoCheckoutAtOptions, RepoCheckoutMode, RepoCheckoutOverwriteMode, RepoMode,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::io::AsRawFd;
use tracing::{error, info, warn};

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;

pub use crate::utils::path_exists;

static PATH_APPS: &str = "/apps";
pub static PATH_REPO_APPS: &str = "/apps/ostree_repo";
static OSTREE_DEPTH: i32 = 1;
static VALIDATE_CHECKOUT: &str = "CheckoutDone";

#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionData {
    pub current_rev: Option<String>,
    pub previous_rev: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn default_from_path() {
        let p = Path::new("./");
        assert_eq!(read_revision_from_file(p), RevisionData::default());
    }

    #[test]
    fn default_from_non_existing_file() {
        let p = Path::new("./bdaskdbkdhiu322");
        assert_eq!(read_revision_from_file(p), RevisionData::default());
    }
    #[test]
    fn default_to_and_from_non_existing_file() {
        let p = Path::new("./testfile");
        let revision = RevisionData {
            current_rev: Some("abcde".to_string()),
            previous_rev: None,
        };
        write_revision_to_file(p, &revision);
        assert_eq!(&read_revision_from_file(p), &revision);
        fs::remove_file(p).unwrap();
    }

    #[test]
    fn test_create_repo_in_empty_directory() {
        let tmp_dir = TempDir::new("example").unwrap();
        create_repo(tmp_dir.path().to_str().unwrap());
        // For now check if there are files inside.
        assert!(!path_is_empty(tmp_dir.path().to_str().unwrap()));
        tmp_dir.close().unwrap();
    }
}
pub(crate) fn get_unit_path(unit: &str) -> String {
    format!("{PATH_APPS}/{unit}/")
}

pub fn read_revision_from_file<P: AsRef<Path>>(path: P) -> RevisionData {
    match _read_revision_from_file(&path) {
        Ok(result) => result,
        Err(error) => {
            error!(
                "Problem opening or reading the file {}: {:?}",
                path.as_ref().display(),
                error
            );
            RevisionData::default()
        }
    }
}

fn write_revision_to_file<P: AsRef<Path>>(path: P, data: &RevisionData) {
    // Open the file in read-only mode with buffer.
    let file = File::create(path).unwrap();

    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &data).unwrap();
}

fn _read_revision_from_file<P: AsRef<Path>>(path: P) -> Result<RevisionData, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;

    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `RevisionData`.
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

#[derive(Debug)]
pub struct ChunkMetaData {
    pub rev: Option<String>,
    //pub autostart: bool,
    //pub autoremove: bool,
    //pub notify: bool,
    //pub timeout: u32,
}

// Returns a ostree user repo from a given directory-
pub fn get_repo(path: &str) -> ostree::Repo {
    create_repo(path);
    let repo = ostree::Repo::new_for_path(path);
    repo.open(None::<&Cancellable>).unwrap();
    repo
}

fn create_repo(path: &str) {
    if !path_exists(path) || path_is_empty(path) {
        info!("Create new repo at {}", path);
        ostree::Repo::create_at(
            libc::AT_FDCWD,
            path,
            RepoMode::BareUserOnly,
            None,
            None::<&Cancellable>,
        )
        .unwrap();
    }
}
fn pull_ostree_ref(_is_container: bool, metadata: &ChunkMetaData, name: &str) {
    let rev = {
        match &metadata.rev {
            None => {
                warn!("No revision found. Using \"{}\" instead", name);
                name
            }
            Some(string) => string,
        }
    };
    let progress = AsyncProgress::new();
    //TBD: Currently the default function pull_default_console_progress_changed
    //     is not available fro ostree-rs. Don't use it for now.
    //progress.connect("changed", true, ostree::Repo::pull_default_console_progress_changed());

    // For options see: https://lazka.github.io/pgi-docs/OSTree-1.0/classes/Repo.html#OSTree.Repo.pull_with_options
    let options = VariantDict::default();
    let flags = ostree::RepoPullFlags::NONE;
    let flags = flags.bits() as i32;
    let flags = flags.to_variant();
    options.insert_value("flags", &flags);
    let depth = OSTREE_DEPTH.to_variant();
    options.insert_value("depth", &depth);
    let refs: &str = rev;
    let array = vec![name, refs].to_variant();
    options.insert_value("refs", &array);
    let options = options.end();

    info!("Upgrader pulling {} from OSTree repo ({})", name, refs);
    let repo_container = get_repo(PATH_REPO_APPS);
    repo_container
        .pull_with_options(name, &options, Some(&progress), None::<&Cancellable>)
        .unwrap();
    progress.finish();
    info!("Upgrader pulled {} from OSTree repo ({})", name, refs);
}

fn get_application_path(name: &str) -> String {
    format!("{PATH_APPS}/{name}")
}

fn get_validation_file(name: &str) -> String {
    format!("{}/{VALIDATE_CHECKOUT}", get_application_path(name))
}

pub(crate) fn checkout_container(metadata: &ChunkMetaData, name: &str) {
    let rev = {
        match &metadata.rev {
            None => return,
            Some(string) => string,
        }
    };
    let options = RepoCheckoutAtOptions {
        overwrite_mode: RepoCheckoutOverwriteMode::UnionIdentical,
        process_whiteouts: true,
        bareuseronly_dirs: true,
        no_copy_fallback: true,
        mode: RepoCheckoutMode::User,
        ..Default::default()
    };
    let repo_container = get_repo(PATH_REPO_APPS);
    let destination_path = get_application_path(name);
    let validation_file = get_validation_file(name);
    let mut revisions = read_revision_from_file(&validation_file);
    revisions.previous_rev = revisions.current_rev;
    revisions.current_rev = Some(rev.to_string());
    if path_exists(&destination_path) {
        info!("Remove application directory {}", &destination_path);
        fs::remove_dir_all(&destination_path).unwrap();
    }
    info!("Create application directory {}", &destination_path);

    fs::create_dir_all(&destination_path).unwrap();
    let dirfd = openat::Dir::open(&destination_path).expect("openat");
    repo_container
        .checkout_at(
            Some(&options),
            dirfd.as_raw_fd(),
            &destination_path,
            rev,
            None::<&Cancellable>,
        )
        .unwrap();
    info!(
        "Checked out application directory {} with revision ({})",
        destination_path, rev
    );
    write_revision_to_file(validation_file, &revisions);
}

pub fn update_container(name: &str, metadata: ChunkMetaData, options: &OstreeOpts) {
    init_container_remote(name.to_string(), options).unwrap();
    pull_ostree_ref(true, &metadata, name);
    checkout_container(&metadata, name);
}

pub(crate) struct Applications(Vec<String>);

impl Applications {
    pub(crate) fn new() -> Applications {
        info!("Getting refs from repo:{}", PATH_REPO_APPS);

        let repo_container = get_repo(PATH_REPO_APPS);
        let refs = repo_container
            .list_refs(None, None::<&Cancellable>)
            .unwrap();
        info!("refs {:#?}", refs);
        info!("There are {} containers.", refs.keys().len());

        let mut a = Applications(Vec::new());
        for key in refs.keys() {
            a.add(key.split(':').next_back().unwrap().to_string());
        }
        a
    }

    fn add(&mut self, elem: String) {
        self.0.push(elem);
    }
}
pub(crate) fn application_exists(name: String) -> bool {
    path_exists(&get_validation_file(&name))
}

impl Default for Applications {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Applications {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

fn init_container_remote(container_name: String, options: &OstreeOpts) -> Result<(), ()> {
    // If the container does not exist, initialize its remote.

    let r_options = VariantDict::default();
    r_options.insert_value("gpg-verify", &options.ostree_gpg_verify.to_variant());
    let r_options = &r_options.end();
    let r_options = Some(r_options);

    let repo_container = get_repo(PATH_REPO_APPS);
    let remote_list = repo_container.remote_list();
    if !remote_list
        .iter()
        .map(|i| i.as_str())
        .any(|x| x == container_name.as_str())
    {
        info!(
            "New container added to the target, we install the remote: {}",
            container_name
        );
        ostree::Repo::remote_add(
            &repo_container,
            container_name.as_ref(),
            Some(options.hostname.as_ref()),
            r_options,
            None::<&Cancellable>,
        )
        .unwrap();
    } else {
        let url = repo_container
            .remote_get_url(container_name.as_str())
            .unwrap();
        let url = url.as_str();
        if options.hostname == url {
            info!(
                "reusing remote: {:#?} url: {:#?}",
                &container_name.as_str(),
                url
            );
        } else {
            info!(
                "For remote {}, {} was expected and {} was received. Replace it.",
                container_name, options.hostname, url
            );
            ostree::Repo::remote_delete(
                &repo_container,
                container_name.as_ref(),
                None::<&Cancellable>,
            )
            .unwrap();
            ostree::Repo::remote_add(
                &repo_container,
                container_name.as_ref(),
                Some(options.hostname.as_ref()),
                r_options,
                None::<&Cancellable>,
            )
            .unwrap();
            info!(
                "Changed url for remote {} to {}.",
                container_name, options.hostname
            );
        }
    }
    Ok(())
}
