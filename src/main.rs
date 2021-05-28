extern crate clap;
extern crate hawkbit;
extern crate ini;
use hawkbit::ddi::{Client, Execution, Finished};
use ini::{Ini, Properties};
use ostree::RepoMode;
use serde::Serialize;
use tokio::time::sleep;

use std::fs;

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
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConfigData {
    #[serde(rename = "HwRevision")]
    hw_revision: String,
}

#[derive(Debug)]
struct HawkbitOpts {
    hostname: String,
    ssl: bool,
    tenant_id: String,
    target_name: String,
    auth_token: String,
    hawkbit_vendor_name: String,
    log_level: String,
}

#[derive(Debug)]
struct OstreeOpts {
    ostree_name_remote: String,
    ostree_gpg_verify: bool,
    ostree_ssl: bool,
    ostree_url_port: String,
    ostreepush_ssh_port: String,
    ostreepush_ssh_user: String,
    ostreepush_ssh_pwd: String,
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let opts: Opts = Opts::parse();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    println!("Value for config: {}", opts.config);

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    match opts.verbose {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        3 | _ => println!("Don't be crazy"),
    }

    let conf = Ini::load_from_file(opts.config).unwrap();

    let section = conf.section(Some("server")).unwrap();
    let server_host_name = get_ini_string(section, "server_host_name");

    let section = conf.section(Some("client")).unwrap();
    let hawkbit_url_port = get_ini_string(section, "hawkbit_url_port");

    let hawkbit_opts: HawkbitOpts = HawkbitOpts {
        hawkbit_vendor_name: get_ini_string(section, "hawkbit_vendor_name"),
        ssl: get_ini_bool(section, "hawkbit_ssl"),
        tenant_id: get_ini_string(section, "hawkbit_tenant_id"),
        target_name: get_ini_string(section, "hawkbit_target_name"),
        auth_token: get_ini_string(section, "hawkbit_auth_token"),
        log_level: get_ini_string(section, "log_level"),
        hostname: format!("http://{}:{}", server_host_name, hawkbit_url_port),
    };
    println!("{:?}", hawkbit_opts);

    let section = conf.section(Some("ostree")).unwrap();
    let ostree_opts: OstreeOpts = OstreeOpts {
        ostree_name_remote: get_ini_string(section, "ostree_name_remote"),
        ostree_gpg_verify: get_ini_bool(section, "ostree_gpg-verify"),
        ostree_ssl: get_ini_bool(section, "ostree_ssl"),
        ostree_url_port: get_ini_string(section, "ostree_url_port"),
        ostreepush_ssh_user: get_ini_string(section, "ostreepush_ssh_user"),
        ostreepush_ssh_pwd: get_ini_string(section, "ostreepush_ssh_pwd"),
        ostreepush_ssh_port: get_ini_string(section, "ostreepush_ssh_port"),
    };
    println!("{:?}", ostree_opts);
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
            println!("Uploading config data");
            let data = ConfigData {
                hw_revision: "1.0".to_string(),
            };

            request
                .upload(Execution::Closed, Finished::Success, None, data, vec![])
                .await
                .expect("foo");
        }

        if let Some(update) = reply.update() {
            println!("Pending update");

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
                print!("Retrieving {}\n", chunk.name());
                get_repo(chunk.name());
            }
            //let artifacts = update.download(Path::new("./download/")).await.expect("kkkk");
            //dbg!(&artifacts);

            //#[cfg(feature = "hash-digest")]
            //for artifact in artifacts {
            //    #[cfg(feature = "hash-md5")]
            //    artifact.check_md5().await?;
            //    #[cfg(feature = "hash-sha1")]
            //    artifact.check_sha1().await?;
            //    #[cfg(feature = "hash-sha256")]
            //    artifact.check_sha256().await?;
            //}

            update
                .send_feedback(Execution::Closed, Finished::Success, vec![])
                .await
                .expect("fff");
        }

        let t = reply.polling_sleep().expect("fff");
        println!("sleep for {:?}", t);
        sleep(t).await;
    }
}
