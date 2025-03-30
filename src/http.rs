use std::net::IpAddr;
use std::os::unix::io::AsRawFd;
use std::result::Result::Ok;

use anyhow::{Context, Result};
use smoltcp::iface::Interface;
use smoltcp::iface::{Config, SocketSet};
use smoltcp::phy;
use smoltcp::phy::TunTapInterface;
use smoltcp::socket::tcp;
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address};
use url::Url;

#[derive(Debug)]
enum HttpState {
    Connect,
    Request,
    Response,
}

fn random_port() -> u16 {
    49152 + rand::random::<u16>() % 16384
}

pub fn get(mut tap: TunTapInterface, mac: EthernetAddress, addr: IpAddr, url: Url) -> Result<()> {
    let domain_name = url.host_str().context("invalid url")?;

    let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
    let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0; 1024]);
    let tcp_socket = tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer);

    let fd = tap.as_raw_fd();
    let default_gateway = Ipv4Address::new(192, 168, 42, 100);
    let mut iface = Interface::new(Config::new(mac.into()), &mut tap, Instant::now());
    iface.update_ip_addrs(|addrs| {
        addrs
            .push(IpCidr::new(IpAddress::v4(192, 168, 42, 1), 24))
            .unwrap();
    });
    iface
        .routes_mut()
        .add_default_ipv4_route(default_gateway)
        .unwrap();

    let mut sockets = SocketSet::new(vec![]);
    let tcp_handle = sockets.add(tcp_socket);

    let http_header = format!(
        "GET {} HTTP/1.0\r\nHost: {}\r\nConnection: close\r\n\r\n",
        url.path(),
        domain_name
    );

    let mut state = HttpState::Connect;
    'http: loop {
        let timestamp = Instant::now();
        iface.poll(timestamp, &mut tap, &mut sockets);

        {
            let socket = sockets.get_mut::<tcp::Socket>(tcp_handle);
            let ctx = iface.context();

            state = match state {
                HttpState::Connect if !socket.is_active() => {
                    eprintln!("connecting");
                    socket.connect(ctx, (addr, 80), random_port())?;
                    HttpState::Request
                }

                HttpState::Request if socket.may_send() => {
                    eprintln!("sending request");
                    socket.send_slice(http_header.as_ref())?;
                    HttpState::Response
                }

                HttpState::Response if socket.can_recv() => {
                    socket.recv(|data| {
                        let output = String::from_utf8_lossy(data);
                        println!("{}", output);
                        (data.len(), 0)
                    })?;
                    HttpState::Response
                }

                HttpState::Response if !socket.may_recv() => {
                    eprintln!("received complete response");
                    break 'http;
                }

                _ => state,
            }
        }
        phy::wait(fd, iface.poll_delay(timestamp, &sockets)).expect("wait error");
    }

    Ok(())
}
