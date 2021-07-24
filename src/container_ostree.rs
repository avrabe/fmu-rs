use gio::NONE_CANCELLABLE;
use glib::prelude::*; // or `use gtk::prelude::*;`
use glib::VariantDict;
use ostree::AsyncProgressExt;
use ostree::RepoMode;
use ostree::*;
use ostree_ext::variant_utils;
use std::fs;
use std::os::unix::io::AsRawFd;
use tracing::info;

pub use crate::utils::path_exists;

static PATH_APPS: &str = "/apps";
static PATH_REPO_APPS: &str = "/apps/ostree_repo";
static OSTREE_DEPTH: i32 = 1;
static VALIDATE_CHECKOUT: &str = "CheckoutDone";

#[derive(Debug)]
pub struct ChunkMetaData {
    pub rev: Option<String>,
    pub autostart: bool,
    pub autoremove: bool,
    pub notify: bool,
    pub timeout: u32,
}

#[derive(Debug)]
pub struct OstreeOpts {
    pub hostname: String,
    pub ostree_name_remote: String,
    pub ostree_gpg_verify: bool,
    pub ostreepush_ssh_port: String,
    pub ostreepush_ssh_user: String,
    pub ostreepush_ssh_pwd: String,
}

// Returns a ostree user repo from a given directory
pub fn get_repo(path: &str) -> ostree::Repo {
    if !path_exists(path) {
        info!("Create new repo at {}", path);
        ostree::Repo::create_at(
            libc::AT_FDCWD,
            path,
            RepoMode::BareUserOnly,
            None,
            gio::NONE_CANCELLABLE,
        )
        .unwrap();
    }
    let repo = ostree::Repo::new_for_path(path);
    ostree::Repo::open(&repo, gio::NONE_CANCELLABLE).unwrap();
    repo
}

fn pull_ostree_ref(_is_container: bool, metadata: &ChunkMetaData, name: &str) {
    let rev = {
        match &metadata.rev {
            None => return,
            Some(string) => string,
        }
    };
    let progress = ostree::AsyncProgress::new();
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
    let array = variant_utils::new_variant_as(&[refs]);
    options.insert_value("refs", &array);
    let options = options.end();

    info!("Upgrader pulling {} from OSTree repo ({})", name, refs);
    let repo_container = get_repo(PATH_REPO_APPS);
    repo_container
        .pull_with_options(name, &options, Some(&progress), gio::NONE_CANCELLABLE)
        .unwrap();
    progress.finish();
    info!("Upgrader pulled {} from OSTree repo ({})", name, refs);
}

fn checkout_container(metadata: &ChunkMetaData, name: &str) {
    let rev = {
        match &metadata.rev {
            None => return,
            Some(string) => string,
        }
    };
    // TODO: stop systemd units.
    let options = RepoCheckoutAtOptions {
        overwrite_mode: RepoCheckoutOverwriteMode::UnionIdentical,
        process_whiteouts: true,
        bareuseronly_dirs: true,
        no_copy_fallback: true,
        mode: RepoCheckoutMode::User,
        ..Default::default()
    };
    let repo_container = get_repo(PATH_REPO_APPS);
    let destination_path = format!("{}/{}", PATH_APPS, name);
    let validation_file = format!("{}/{}", &destination_path, VALIDATE_CHECKOUT);
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
            gio::NONE_CANCELLABLE,
        )
        .unwrap();
    info!(
        "Checked out application directory {} with revision ({})",
        destination_path, rev
    );
    fs::File::create(validation_file).unwrap();
}

pub fn update_container(name: &str, metadata: ChunkMetaData, options: &OstreeOpts) {
    init_container_remote(name.to_string(), options).unwrap();
    pull_ostree_ref(true, &metadata, name);
    checkout_container(&metadata, name);
}

pub fn init_checkout_existing_containers() {
    info!("Getting refs from repo:{}", PATH_REPO_APPS);

    let repo_container = get_repo(PATH_REPO_APPS);
    let refs = repo_container.list_refs(None, NONE_CANCELLABLE).unwrap();
    ////     [_, refs] = self.repo_containers.list_refs(None, None)
    info!("refs {:#?}", refs);
    info!("There are {} containers to be started.", refs.keys().len());
    ////     self.logger.info("There are {} containers to be started.".format(len(refs)))
    //     for ref in refs:
    //         container_name = ref.split(':')[1]
    //         if not os.path.isfile(PATH_APPS + '/' + container_name + '/' + VALIDATE_CHECKOUT):
    //             self.checkout_container(container_name, None)
    //             self.update_container_ids(container_name)
    //         if not res:
    //             self.logger.error("Error when checking out container:{}".format(container_name))
    //             break
    //         self.create_unit(container_name)
    //     self.systemd.Reload()
    //     for ref in refs:
    //         container_name = ref.split(':')[1]
    //         if os.path.isfile(PATH_APPS + '/' + container_name + '/' + FILE_AUTOSTART):
    //             self.start_unit(container_name)
    // except (GLib.Error, Exception) as e:
    //     self.logger.error("Error checking out containers repo ({})".format(e))
    //     res = False
    // finally:
    //     return res
}

pub fn init_ostree_remotes(options: &OstreeOpts) -> Result<(), ()> {
    //// res = True
    let repo_container = get_repo(PATH_REPO_APPS);

    // self.ostree_remote_attributes = ostree_remote_attributes
    // opts = GLib.Variant('a{sv}', {'gpg-verify': GLib.Variant('b', ostree_remote_attributes['gpg-verify'])})
    // try:
    //     self.logger.info("Initalize remotes for the OS ostree: {}".format(ostree_remote_attributes['name']))
    //     if not ostree_remote_attributes['name'] in self.repo_os.remote_list():
    //         self.repo_os.remote_add(ostree_remote_attributes['name'],
    //                                 ostree_remote_attributes['url'],
    //                                 opts, None)
    //     self.remote_name_os = ostree_remote_attributes['name']

    let refs = repo_container
        .list_refs(None, gio::NONE_CANCELLABLE)
        .unwrap();
    info!(
        "Initalize remotes for the containers ostree: {:#?}",
        refs.keys()
    );
    let remote_list = repo_container.remote_list();
    let remote_list: Vec<&str> = remote_list.iter().map(|i| i.as_str()).collect();
    info!("remote_list {:#?}", remote_list);
    for remote in remote_list.iter() {
        let url = repo_container.remote_get_url(remote).unwrap();
        let url = url.as_str();
        info!("remote: {:#?} url: {:#?}", remote, url);
        if options.hostname == url {
            info!("reusing remote: {:#?} url: {:#?}", remote, url);
            // TODO ^ Try deleting the & and matching just "Ferris"
        } else {
            info!(
                "For remote {}, {} was expected and {} was received",
                remote, options.hostname, url
            );
        }
    }

    //         if remote_name not in self.repo_containers.remote_list():
    //             self.logger.info("We had the remote: {}".format(remote_name))
    //             self.repo_containers.remote_add(remote_name,
    //                                             ostree_remote_attributes['url'],
    //                                             opts, None)

    Ok(())
}

fn init_container_remote(container_name: String, options: &OstreeOpts) -> Result<(), ()> {
    // """
    // If the container does not exist, initialize its remote.

    // Parameters:
    // container_name (str): name of the container
    // """

    // # returns [('container-hello-world.service', 'description', 'loaded', 'failed', 'failed', '', '/org/freedesktop/systemd1/unit/wtk_2dnodejs_2ddemo_2eservice', 0, '', '/')]
    // service = self.systemd.ListUnitsByNames([container_name + '.service'])

    // try:
    //     if (service[0][2] == 'not-found'):
    //         # New service added, we need to connect to its remote
    //         opts = GLib.Variant('a{sv}',
    //                             {'gpg-verify': GLib.Variant('b', self.ostree_remote_attributes['gpg-verify'])})
    //         # Check if this container was not installed previously
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
            options.hostname.as_ref(),
            None,
            gio::NONE_CANCELLABLE,
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
                gio::NONE_CANCELLABLE,
            )
            .unwrap();
            ostree::Repo::remote_add(
                &repo_container,
                container_name.as_ref(),
                options.hostname.as_ref(),
                None,
                gio::NONE_CANCELLABLE,
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
