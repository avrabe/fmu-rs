extern crate clap;
extern crate hawkbit;
extern crate ini;
use crate::container_ostree::application_exists;
use crate::container_ostree::checkout_container;
use crate::container_ostree::get_unit_path;
use crate::container_ostree::update_container;
use crate::container_ostree::Applications;
use crate::container_ostree::ChunkMetaData;
use crate::ostree::OstreeOpts;
use crate::systemd::{
    create_unit, disable_unit_file, enable_unit_file, reload, start_unit, stop_unit,
};
use clap::{AppSettings, Clap};
use hawkbit::ddi::{Client, Execution, Finished};
use ini::{Ini, Properties};
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

mod container_ostree;
mod ostree;
mod rootfs_ostree;
mod systemd;
mod utils;
pub use crate::utils::path_exists;

pub fn get_ini_string(section: &Properties, name: &str) -> String {
    if let Some(result) = section.get(name) {
        result.to_string()
    } else {
        warn!("Missing configuration entry {}.", name);
        "".to_string()
    }
    //section.get(name).unwrap_or_else(|| info!("ddd"); "").to_string()
}

pub fn get_ini_bool(section: &Properties, name: &str) -> bool {
    if let Some(result) = section.get(name) {
        result.parse().unwrap()
    } else {
        warn!("Missing configuration entry {}.", name);
        false
    }
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

    let ini_log_level = read_loglevel_configuration(&conf);

    let log_level = if opts.debug {
        "debug".to_string()
    } else {
        ini_log_level
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

    // Read the server information
    let server_host_name = read_server_configuration(&conf);
    let hawkbit_opts = read_hawkbit_configuration(&conf, &server_host_name);
    // setup ostree
    let ostree_opts = read_ostree_configuration(conf, server_host_name);

    // more program logic goes here...
    let applications = Applications::new();
    for application in applications.into_iter() {
        if !application_exists(application.to_string()) {
            let chunk_meta_data: ChunkMetaData = ChunkMetaData {
                rev: Some(format!(
                    "{}:{}",
                    application.to_string(),
                    application.to_string()
                )),
                ..Default::default()
            };
            checkout_container(&chunk_meta_data, &application);
        }
        create_unit(&application, &get_unit_path(&application));
        enable_unit_file(&application, false, false);
        reload();
        start_unit(&application);
    }

    let ddi = Client::new(
        &hawkbit_opts.hostname,
        &hawkbit_opts.tenant_id,
        &hawkbit_opts.target_name,
        &hawkbit_opts.auth_token,
    )
    .unwrap();

    loop {
        if let Ok(reply) = ddi.poll().await {
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
                    create_unit(unit, &get_unit_path(unit));
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
        } else {
            warn!("Problems while connecting to the server");
            let t: Duration = Duration::new(30, 0);
            info!("sleep for {:?}", t);
            sleep(t).await;
        }
        //dbg!(&reply);
    }
}

fn read_loglevel_configuration(conf: &Ini) -> String {
    let section = conf.section(Some("client")).unwrap();
    get_ini_string(section, "log_level")
}

fn read_server_configuration(conf: &Ini) -> String {
    let section = conf.section(Some("server")).unwrap();
    get_ini_string(section, "server_host_name")
}

fn read_hawkbit_configuration(conf: &Ini, server_host_name: &str) -> HawkbitOpts {
    let section = conf.section(Some("client")).unwrap();
    let hawkbit_url_port = get_ini_string(section, "hawkbit_url_port");
    let hawkbit_ssl = get_ini_bool(section, "hawkbit_ssl");
    let hawkbit_url_type = if hawkbit_ssl { "https://" } else { "http://" };
    let hawkbit_opts: HawkbitOpts = HawkbitOpts {
        tenant_id: get_ini_string(section, "hawkbit_tenant_id"),
        target_name: get_ini_string(section, "hawkbit_target_name"),
        auth_token: get_ini_string(section, "hawkbit_auth_token"),
        hostname: format!(
            "{}{}:{}",
            hawkbit_url_type, server_host_name, hawkbit_url_port
        ),
    };
    info!("{:?}", hawkbit_opts);
    hawkbit_opts
}

fn read_ostree_configuration(conf: Ini, server_host_name: String) -> OstreeOpts {
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
    ostree_opts
}
