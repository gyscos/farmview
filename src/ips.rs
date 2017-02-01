use std::str::FromStr;

pub struct IpBlock {
    netmask: u32,
    network: u32,
}

impl IpBlock {
    fn parse(ip: &str) -> u32 {
        ip.split('.')
            .take(4)
            .map(|n| u8::from_str(n).unwrap_or(0))
            .enumerate()
            .map(|(i, n)| (n as u32) << (8 * (3 - i as u32)))
            .fold(0, |a, b| a | b)
    }

    pub fn new(range: &str) -> Self {
        let tokens: Vec<_> = range.split('/').collect();
        let ip = Self::parse(tokens[0]);
        // println!("Net: {:b}", ip);
        let bits = 32 - u8::from_str(tokens[1]).unwrap_or(32);
        let netmask = (!0u32 >> bits) << bits;
        // println!("netmask: {:b}", netmask);

        IpBlock {
            netmask: netmask,
            network: ip & netmask,
        }
    }

    pub fn matches(&self, ip: &str) -> bool {
        let ip = Self::parse(ip);
        // println!("IP: {:b}", ip);
        (ip & self.netmask) == self.network
    }
}
