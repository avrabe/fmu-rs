extern crate clap;
extern crate hawkbit;
extern crate ini;
use hawkbit::ddi::{Client, Execution, Finished};
use ini::{Ini, Properties};
use ostree::RepoMode;
use serde::Serialize;
use std::fs;
use tokio::time::sleep;

use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub fn path_exists(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_path() {
        assert_eq!(path_exists("./"), true);
    }

    #[test]
    fn bad_path() {
        assert_eq!(path_exists("./sadsad/"), false);
    }
}
pub fn get_ini_string(section: &Properties, name: &str) -> String {
    section.get(name).unwrap().to_string()
}

pub fn get_ini_bool(section: &Properties, name: &str) -> bool {
    section.get(name).unwrap().parse().unwrap()
}
pub fn get_repo(path: &str) -> ostree::Repo {
    //let options = RepoCheckoutAtOptions {
    //    mode: RepoCheckoutMode::User,
    //    overwrite_mode: RepoCheckoutOverwriteMode::UnionIdentical,
    //    process_whiteouts: true,
    //    bareuseronly_dirs: true,
    //    no_copy_fallback: true,
    //
    //    force_copy: true,
    //    enable_uncompressed_cache: false,
    //    enable_fsync: false,
    //    force_copy_zerosized: false,
    //    subpath: None,
    //    devino_to_csum_cache: None,
    //    filter: None,
    //    sepolicy: None,
    //    sepolicy_prefix: None,
    //};
    //let o: Option<&RepoCheckoutAtOptions> = Some(&options);
    //let options = ostree::RepoCheckoutAtOptions {
    //    mode:
    //}
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
// (Full example with detailed comments in examples/01d_quick_example.rs)
//
// This example demonstrates clap's full 'custom derive' style of creating arguments which is the
// simplest method of use, but sacrifices some flexibility.
use clap::{AppSettings, Clap};

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "me")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "config.cfg")]
    config: String,
    /// Print debug information
    #[clap(short)]
    debug: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConfigData {
    #[serde(rename = "HwRevision")]
    hw_revision: String,
}

#[derive(Debug)]
struct HawkbitOpts {
    hostname: String,
    tenant_id: String,
    target_name: String,
    auth_token: String,
    hawkbit_vendor_name: String,
}

#[derive(Debug)]
struct OstreeOpts {
    hostname: String,
    ostree_name_remote: String,
    ostree_gpg_verify: bool,
    ostreepush_ssh_port: String,
    ostreepush_ssh_user: String,
    ostreepush_ssh_pwd: String,
}

pub fn get_log_level(level: &str) -> Level {
    match level.to_uppercase().as_ref() {
        "DEBUG" => Level::DEBUG,
        "INFO" => Level::INFO,
        "WARN" => Level::WARN,
        "ERROR" => Level::ERROR,
        "FATAL" => Level::ERROR,
        _ => Level::INFO,
    }
}

//static PATH_APPS: &str = "/apps";
static PATH_REPO_APPS: &str = "/apps/ostree_repo";

fn init_checkout_existing_containers() {
    // res = True
    info!("Getting refs from repo:{}", PATH_REPO_APPS);
    //// self.logger.info("Getting refs from repo:{}".format(PATH_REPO_APPS))

    // try:
    let repo_container = get_repo(PATH_REPO_APPS);
    let refs = repo_container
        .list_refs(None, gio::NONE_CANCELLABLE)
        .unwrap();
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

fn init_ostree_remotes(options: &OstreeOpts) -> Result<(), ()> {
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
    //     [_, refs] = self.repo_containers.list_refs(None, None)
    info!(
        "Initalize remotes for the containers ostree: {:#?}",
        refs.keys()
    );
    //     self.logger.info("Initalize remotes for the containers ostree: {}".format(refs))
    //     for ref in refs:
    //         remote_name = ref.split(':')[0]
    let remote_list = repo_container.remote_list();
    let remote_list: Vec<&str> = remote_list.iter().map(|i| i.as_str()).collect();
    info!("remote_list {:#?}", remote_list);

    //         if remote_name not in self.repo_containers.remote_list():
    //             self.logger.info("We had the remote: {}".format(remote_name))
    //             self.repo_containers.remote_add(remote_name,
    //                                             ostree_remote_attributes['url'],
    //                                             opts, None)

    // except GLib.Error as e:
    //     self.logger.error("OSTRee remote initialization failed ({})".format(str(e)))
    //     res = False

    // return res

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
    let remote_list: Vec<&str> = remote_list.iter().map(|i| i.as_str()).collect();
    if !remote_list.contains(&container_name.as_str()) {
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
        info!(
            "New container {} added to the target but the remote already exists, we do nothing",
            container_name
        );
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    // Get the command line arguments
    let opts: Opts = Opts::parse();

    // Read from the configuration file
    let conf = Ini::load_from_file(&opts.config).unwrap();

    // Read the server information
    let section = conf.section(Some("server")).unwrap();
    let server_host_name = get_ini_string(section, "server_host_name");

    let section = conf.section(Some("client")).unwrap();
    let log_level = if opts.debug {
        "debug".to_string()
    } else {
        get_ini_string(section, "log_level")
    };

    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(get_log_level(&log_level))
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    info!("Value for config: {}", opts.config);
    info!("LogLevel: {}", log_level);

    let hawkbit_url_port = get_ini_string(section, "hawkbit_url_port");
    let hawkbit_ssl = get_ini_bool(section, "hawkbit_ssl");

    let hawkbit_url_type = if hawkbit_ssl { "https://" } else { "http://" };
    // setup hawkbit
    let hawkbit_opts: HawkbitOpts = HawkbitOpts {
        hawkbit_vendor_name: get_ini_string(section, "hawkbit_vendor_name"),
        tenant_id: get_ini_string(section, "hawkbit_tenant_id"),
        target_name: get_ini_string(section, "hawkbit_target_name"),
        auth_token: get_ini_string(section, "hawkbit_auth_token"),
        hostname: format!(
            "{}{}:{}",
            hawkbit_url_type, server_host_name, hawkbit_url_port
        ),
    };
    info!("{:?}", hawkbit_opts);

    // setup ostree
    let section = conf.section(Some("ostree")).unwrap();
    let ostree_ssl = get_ini_bool(section, "ostree_ssl");
    let ostree_url_port = get_ini_string(section, "ostree_url_port");
    let ostree_url_type = if ostree_ssl { "https://" } else { "http://" };
    let ostree_url_prefix = get_ini_string(section, "ostree_url_prefix");

    let ostree_opts: OstreeOpts = OstreeOpts {
        hostname: format!(
            "{}{}:{}/{}",
            ostree_url_type, server_host_name, ostree_url_port, ostree_url_prefix
        ),
        ostree_name_remote: get_ini_string(section, "ostree_name_remote"),
        ostree_gpg_verify: get_ini_bool(section, "ostree_gpg-verify"),
        ostreepush_ssh_user: get_ini_string(section, "ostreepush_ssh_user"),
        ostreepush_ssh_pwd: get_ini_string(section, "ostreepush_ssh_pwd"),
        ostreepush_ssh_port: get_ini_string(section, "ostreepush_ssh_port"),
    };
    info!("{:?}", ostree_opts);
    // more program logic goes here...
    init_checkout_existing_containers();
    init_ostree_remotes(&ostree_opts).unwrap();
    init_container_remote("bar".to_string(), &ostree_opts).unwrap();
    //let _repo = &ostree::Repo::checkout_at(&repo, o, libc::AT_FDCWD, "./download/", "init", gio::NONE_CANCELLABLE);

    let ddi = Client::new(
        &hawkbit_opts.hostname,
        &hawkbit_opts.tenant_id,
        &hawkbit_opts.target_name,
        &hawkbit_opts.auth_token,
    )
    .unwrap();

    loop {
        let reply = ddi.poll().await.expect("buh");
        dbg!(&reply);

        if let Some(request) = reply.config_data_request() {
            info!("Uploading config data");
            let data = ConfigData {
                hw_revision: "1.0".to_string(),
            };

            request
                .upload(Execution::Closed, Finished::Success, None, data, vec![])
                .await
                .expect("foo");
        }

        if let Some(update) = reply.update() {
            info!("Pending update");

            let update = update.fetch().await;
            let update = match update {
                Ok(update) => update,
                Err(error) => panic!("Problem opening the file: {:?}", error),
            };

            dbg!(&update);

            update
                .send_feedback(Execution::Proceeding, Finished::None, vec!["Downloading"])
                .await
                .expect("ff");

            for chunk in update.chunks() {
                //info!("Retrieving {}\n", chunk.name());
                info!("metadata {:#?}", chunk.metadata());
                get_repo(chunk.name());
            }

            update
                .send_feedback(Execution::Closed, Finished::Success, vec![])
                .await
                .expect("fff");
        }

        let t = reply.polling_sleep().expect("fff");
        info!("sleep for {:?}", t);
        sleep(t).await;
    }
}
