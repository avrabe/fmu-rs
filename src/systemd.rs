extern crate rustbus;
use rustbus::{
    connection::{rpc_conn::RpcConn, Timeout},
    get_system_bus_path,
    message_builder::MarshalledMessage,
    DuplexConn,
};
use std::fs;
use tracing::info;

static PATH_SYSTEMD_UNITS: &str = "/etc/systemd/system/";

pub(crate) fn create_unit(unit: &str, unit_path: &str) {
    let destination = format!("{PATH_SYSTEMD_UNITS}{unit}.service");
    let source = format!("{unit_path}/systemd.service");
    fs::copy(source, destination).unwrap();
}

pub(crate) fn disable_unit_file(unit: &str, runtime: bool) {
    let unit = &(format!("{unit}.service"));
    info!("disabling unit {}", unit);
    let (rpc_conn, mut msg) = create_manager("DisableUnitFiles");
    let units = vec![unit];
    msg.body.push_param(units).unwrap();
    msg.body.push_param(runtime).unwrap();
    send_message(rpc_conn, msg);
    info!("disabled unit {}", unit);
}

pub(crate) fn enable_unit_file(unit: &str, runtime: bool, force: bool) {
    let unit = &(format!("{unit}.service"));
    info!("enabling unit {}", unit);
    let (rpc_conn, mut msg) = create_manager("EnableUnitFiles");
    let units = vec![unit];
    msg.body.push_param(units).unwrap();
    msg.body.push_param(runtime).unwrap();
    msg.body.push_param(force).unwrap();
    send_message(rpc_conn, msg);
    info!("enabled unit {}", unit);
}

pub(crate) fn start_unit(unit: &str) {
    info!("starting unit {}", unit);
    startstop_manager("StartUnit", unit);
    info!("started unit {}", unit);
}

pub(crate) fn stop_unit(unit: &str) {
    info!("stopping unit {}", unit);
    startstop_manager("StopUnit", unit);
    info!("stopped unit {}", unit);
}

pub(crate) fn reload() {
    info!(r#"reloading systemd"#);
    let (rpc_conn, msg) = create_manager("Reload");
    send_message(rpc_conn, msg);
    info!("reloaded systemd");
}

fn wait_response(ctx: u32, mut rpc_conn: RpcConn) {
    let resp = rpc_conn
        .wait_response(ctx, rustbus::connection::Timeout::Infinite)
        .unwrap();

    if let rustbus::message_builder::MessageType::Error = resp.typ {
        println!(
            "Error name: {}",
            resp.dynheader.error_name.as_ref().unwrap()
        );
        println!("Error: {}", resp.body.parser().get::<&str>().unwrap());
    } else {
        info!("No error message received. Response: {:?}", resp);
    };
}
fn startstop_manager(member: &str, unit: &str) {
    let unit = &(format!("{unit}.service"));
    let (rpc_conn, mut msg) = create_manager(member);
    msg.body.push_param(unit).unwrap();
    msg.body.push_param("replace").unwrap();
    send_message(rpc_conn, msg);
}

fn send_message(mut rpc_conn: RpcConn, mut msg: rustbus::message_builder::MarshalledMessage) {
    let ctx = rpc_conn
        .send_message(&mut msg)
        .unwrap()
        .write_all()
        .unwrap();
    wait_response(ctx, rpc_conn);
}

fn create_manager(member: &str) -> (RpcConn, MarshalledMessage) {
    let system_path = get_system_bus_path().unwrap();
    let mut con = DuplexConn::connect_to_bus(system_path, true).unwrap();
    let _unique_name = con.send_hello(Timeout::Infinite).unwrap();
    let rpc_conn = RpcConn::new(con);
    let msg: rustbus::message_builder::MarshalledMessage =
        rustbus::message_builder::MessageBuilder::new()
            .call(member)
            .on("/org/freedesktop/systemd1")
            .with_interface("org.freedesktop.systemd1.Manager")
            .at("org.freedesktop.systemd1")
            .build();
    (rpc_conn, msg)
}
