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
pub fn get_repo(path: &str) {
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
        &ostree::Repo::create_at(
            libc::AT_FDCWD,
            path,
            RepoMode::BareUserOnly,
            None,
            gio::NONE_CANCELLABLE,
        )
        .unwrap();
    }
    let repo = &ostree::Repo::new_for_path(path);
    &ostree::Repo::open(repo, gio::NONE_CANCELLABLE);
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

pub fn get_log_level(level: &String) -> Level {
    match level.to_uppercase().as_ref() {
        "DEBUG" => return Level::DEBUG,
        "INFO" => return Level::INFO,
        "WARN" => return Level::WARN,
        "ERROR" => return Level::ERROR,
        "FATAL" => return Level::ERROR,
        _ => return Level::INFO,
    }
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

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
    info!("Value for config: {}", opts.config);
    info!("LogLevel: {}", log_level);

    let hawkbit_url_port = get_ini_string(section, "hawkbit_url_port");
    let hawkbit_ssl= get_ini_bool(section, "hawkbit_ssl");

    let hawkbit_url_type = if hawkbit_ssl {
        "https://"
    } else {
        "http://"
    };
    // setup hawkbit
    let hawkbit_opts: HawkbitOpts = HawkbitOpts {
        hawkbit_vendor_name: get_ini_string(section, "hawkbit_vendor_name"),
        tenant_id: get_ini_string(section, "hawkbit_tenant_id"),
        target_name: get_ini_string(section, "hawkbit_target_name"),
        auth_token: get_ini_string(section, "hawkbit_auth_token"),
        hostname: format!("{}{}:{}", hawkbit_url_type, server_host_name, hawkbit_url_port),
    };
    info!("{:?}", hawkbit_opts);

    // setup ostree
    let section = conf.section(Some("ostree")).unwrap();
    let ostree_ssl= get_ini_bool(section, "ostree_ssl");
    let ostree_url_port= get_ini_string(section, "ostree_url_port");
    let ostree_url_type = if ostree_ssl {
        "https://"
    } else {
        "http://"
    };

    let ostree_opts: OstreeOpts = OstreeOpts {
        hostname: format!("{}{}:{}", ostree_url_type,server_host_name, ostree_url_port),
        ostree_name_remote: get_ini_string(section, "ostree_name_remote"),
        ostree_gpg_verify: get_ini_bool(section, "ostree_gpg-verify"),
        ostreepush_ssh_user: get_ini_string(section, "ostreepush_ssh_user"),
        ostreepush_ssh_pwd: get_ini_string(section, "ostreepush_ssh_pwd"),
        ostreepush_ssh_port: get_ini_string(section, "ostreepush_ssh_port"),
    };
    info!("{:?}", ostree_opts);
    // more program logic goes here...

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
    }; 
}
