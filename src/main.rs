extern crate clap;
extern crate hawkbit;
extern crate ini;
use crate::container_ostree::init_checkout_existing_containers;
use crate::container_ostree::update_container;
use crate::container_ostree::ChunkMetaData;
use crate::ostree::OstreeOpts;
use crate::rootfs_ostree::init_ostree_remotes;
use crate::systemd::{disable_unit_file, enable_unit_file, reload, start_unit, stop_unit};
use clap::{AppSettings, Clap};
use hawkbit::ddi::{Client, Execution, Finished};
use ini::{Ini, Properties};
use serde::Serialize;
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod container_ostree;
mod ostree;
mod rootfs_ostree;
mod systemd;
mod utils;
pub use crate::utils::path_exists;

pub fn get_ini_string(section: &Properties, name: &str) -> String {
    section.get(name).unwrap().to_string()
}

pub fn get_ini_bool(section: &Properties, name: &str) -> bool {
    section.get(name).unwrap().parse().unwrap()
}

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

    let ddi = Client::new(
        &hawkbit_opts.hostname,
        &hawkbit_opts.tenant_id,
        &hawkbit_opts.target_name,
        &hawkbit_opts.auth_token,
    )
    .unwrap();

    loop {
        let reply = ddi.poll().await.expect("buh");
        //dbg!(&reply);

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

            //dbg!(&update);

            update
                .send_feedback(Execution::Proceeding, Finished::None, vec!["Downloading"])
                .await
                .expect("ff");

            for chunk in update.chunks() {
                info!("Retrieving {}\n", chunk.name());
                let mut rev: Option<String> = None;
                let mut autostart: bool = false;
                let mut autoremove: bool = false;
                let mut notify: bool = false;
                let mut timeout: u32 = 0;

                for metadata in chunk.metadata() {
                    match metadata {
                        ("rev", _) => rev = Some(metadata.1.to_string()),
                        ("autostart", _) => autostart = metadata.1 == "1",
                        ("autoremove", _) => autoremove = metadata.1 == "1",
                        ("notify", _) => notify = metadata.1 == "1",
                        ("timeout", _) => timeout = metadata.1.parse().unwrap(),
                        (_, _) => info!("unknown metadata {:#?}", metadata),
                    }
                }
                let chunk_meta_data: ChunkMetaData = ChunkMetaData {
                    rev,
                    autostart,
                    autoremove,
                    notify,
                    timeout,
                };
                info!("metadata: {:#?}", chunk_meta_data);
                let unit = chunk.name();
                stop_unit(unit);
                disable_unit_file(unit, false);
                update_container(unit, chunk_meta_data, &ostree_opts);
                enable_unit_file(unit, false, false);
                reload();
                start_unit(unit);
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
