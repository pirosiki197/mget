use clap::{App, Arg};
use smoltcp::phy::{Medium, TunTapInterface};
use url::Url;

mod dns;
mod ethernet;
mod http;

fn main() {
    let app = App::new("mget")
        .about("GET a Web page, manually")
        .arg(Arg::with_name("url").required(true))
        .arg(Arg::with_name("tap-device").required(true))
        .arg(Arg::with_name("dns-server").default_value("8.8.8.8"))
        .get_matches();

    let url_text = app.value_of("url").unwrap();
    let dns_server_text = app.value_of("dns-server").unwrap();
    let tap_text = app.value_of("tap-device").unwrap();

    let url = Url::parse(url_text).expect("error: unable to parse URL");

    if url.scheme() != "http" {
        eprintln!("error: only HTTP URLs are supported");
        return;
    }

    let tap = TunTapInterface::new(tap_text, Medium::Ethernet)
        .expect("error: unabl to use TAP interface");

    let domain_name = url.host_str().expect("error: domain name required");

    let _dns_server: std::net::Ipv4Addr = dns_server_text
        .parse()
        .expect("error: unable to parse DNS server");

    let addr = dns::resolve(dns_server_text, domain_name).unwrap().unwrap();

    let mac = ethernet::MacAddress::new().into();

    http::get(tap, mac, addr, url).unwrap();
}
