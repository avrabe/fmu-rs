extern crate rustbus;
use rustbus::{connection::Timeout, get_system_bus_path, DuplexConn};

#[cfg(test)]
mod tests {
    use super::*;
    use rustbus::MessageBuilder;

    #[test]

    fn sy_test() {
        systemd_test();
        systemd_start_unit("test");
    }

    #[test]
    fn systemd_test() {
        let system_path = get_system_bus_path().unwrap();
        let mut con = DuplexConn::connect_to_bus(system_path, true).unwrap();
        // Dont forget to send the obligatory hello message. send_hello wraps the call and parses the response for convenience.
        let _unique_name = con.send_hello(Timeout::Infinite).unwrap();

        // Next you will probably want to create a new message to send out to the world
        let mut sig = MessageBuilder::new()
            .signal(
                "io.killing.spark".to_string(),
                "TestSignal".to_string(),
                "/io/killing/spark".to_string(),
            )
            .build();

        // To put parameters into that message you use the sig.body.push_param functions. These accept anything that can be marshalled into a dbus parameter
        // You can derive or manually implement that trait for your own types if you need that.
        sig.body.push_param("My cool new Signal!").unwrap();

        // Now send you signal to all that want to hear it!
        let ctx = con.send.send_message(&sig).unwrap();
        ctx.write_all().unwrap();
    }
}

pub fn systemd_start_unit(unit: &str) {
    let system_path = get_system_bus_path().unwrap();
    let mut con = DuplexConn::connect_to_bus(system_path, true).unwrap();
    // Dont forget to send the obligatory hello message. send_hello wraps the call and parses the response for convenience.
    let _unique_name = con.send_hello(Timeout::Infinite).unwrap();

    let mut rpc_conn = rustbus::connection::rpc_conn::RpcConn::new(con);
    let mut msg = rustbus::message_builder::MessageBuilder::new()
        .call("StartUnit")
        .on("/org/freedesktop/systemd1")
        .with_interface("org.freedesktop.systemd1.Manager")
        .at("org.freedesktop.systemd1")
        .build();
    msg.body.push_param(&unit).unwrap();
    msg.body.push_param(&"start").unwrap();

    rpc_conn
        .send_message(&mut msg)
        .unwrap()
        .write_all()
        .unwrap();
}
