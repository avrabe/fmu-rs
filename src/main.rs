extern crate ini;
extern crate hawkbit;
extern crate clap;
use hawkbit::ddi::{Client};
use ini::Ini;

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


#[tokio::main]
pub async fn main() {
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

    let server_host_name = conf.section(Some("server")).unwrap().get("server_host_name").unwrap();
    let section = conf.section(Some("client")).unwrap();
    let hawkbit_vendor_name = section.get("hawkbit_vendor_name").unwrap();
    let hawkbit_url_port = section.get("hawkbit_url_port").unwrap();
    let ssl = section.get("hawkbit_ssl").unwrap().parse::<bool>().unwrap();
    let tenant_id = section.get("hawkbit_tenant_id").unwrap();
    let target_name = section.get("hawkbit_target_name").unwrap();
    let auth_token = section.get("hawkbit_auth_token").unwrap();
    let log_level = section.get("log_level").unwrap().parse::<log::Level>().unwrap();
    //let foo: Ipv4Addr = tommy.parse::<Ipv4Addr>().unwrap();

    println!("{:?}", server_host_name);
    println!("{:?}", ssl);
    println!("{:?}", tenant_id);
    println!("{:?}", target_name);
    println!("{:?}", auth_token);
    println!("{:?}", hawkbit_vendor_name);
    println!("{:?}", hawkbit_url_port);
    println!("{:?}", log_level);
    // more program logic goes here...

    let ddi = Client::new( &server_host_name,&tenant_id, &target_name, &auth_token ).unwrap();
    let reply = ddi.poll().await;
    dbg!(&reply);

}
