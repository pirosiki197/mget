use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use anyhow::Result;
use hickory_proto::op::{Message, MessageType, OpCode, Query};
use hickory_proto::rr::domain::Name;
use hickory_proto::rr::record_type::RecordType;
use hickory_proto::serialize::binary::*;

fn message_id() -> u16 {
    let candidate = rand::random();
    if candidate == 0 {
        return message_id();
    }
    candidate
}

pub fn resolve(dns_server_address: &str, domain_name: &str) -> Result<Option<std::net::IpAddr>> {
    let domain_name = Name::from_ascii(domain_name)?;

    let dns_server_address = format!("{}:53", dns_server_address);
    let dns_server: SocketAddr = dns_server_address.parse()?;
    let mut request_buffer: Vec<u8> = Vec::with_capacity(64);
    let mut response_buffer: Vec<u8> = vec![0; 512];

    let mut request = Message::new();
    request.add_query(Query::query(domain_name, RecordType::A));
    request
        .set_id(message_id())
        .set_message_type(MessageType::Query)
        .set_op_code(OpCode::Query)
        .set_recursion_desired(true);

    let localhost = UdpSocket::bind("0.0.0.0:0")?;

    let timeout = Duration::from_secs(5);
    localhost.set_read_timeout(Some(timeout))?;
    localhost.set_nonblocking(false)?;

    let mut encoder = BinEncoder::new(&mut request_buffer);
    request.emit(&mut encoder)?;

    let _n_bytes_sent = localhost.send_to(&request_buffer, dns_server)?;

    loop {
        let (_b_bytes_recv, remote_port) = localhost.recv_from(&mut response_buffer)?;
        if remote_port == dns_server {
            break;
        }
    }

    let response = Message::from_vec(&response_buffer)?;

    for answer in response.answers() {
        if answer.record_type() == RecordType::A {
            let resource = answer.data();
            let server_ip = resource.ip_addr().expect("invalid IP address received");
            return Ok(Some(server_ip));
        }
    }

    return Ok(None);
}
