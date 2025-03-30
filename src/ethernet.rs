use rand;
use std::fmt;
use std::fmt::Display;

use rand::RngCore;
use smoltcp::wire;

#[derive(Debug)]
pub struct MacAddress([u8; 6]);

impl Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let octal = self.0;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            octal[0], octal[1], octal[2], octal[3], octal[4], octal[5]
        )
    }
}

impl MacAddress {
    pub fn new() -> MacAddress {
        let mut octets = [0; 6];
        rand::rng().fill_bytes(&mut octets);
        octets[0] |= 0x02; // Set the local bit
        octets[0] &= 0xfe; // Clear the multicast bit
        MacAddress(octets)
    }
}

impl Into<wire::EthernetAddress> for MacAddress {
    fn into(self) -> wire::EthernetAddress {
        wire::EthernetAddress(self.0)
    }
}
