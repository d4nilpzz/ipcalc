use clap::Parser;
use colored::*;
use std::net::Ipv4Addr;
use std::str::FromStr;

#[derive(Parser)]
#[command(author = "d4nilpzz", version = "1.0.0", about = "ipcalc - simple IPv4/CIDR calculator")]
struct Args {
    input: String,
}

fn ipv4_to_u32(a: Ipv4Addr) -> u32 {
    u32::from_be_bytes(a.octets())
}
fn u32_to_ipv4(x: u32) -> Ipv4Addr {
    Ipv4Addr::from(x.to_be_bytes())
}
fn mask_from_prefix(prefix: u8) -> u32 {
    if prefix == 0 {
        0
    } else {
        (!0u32) << (32 - prefix)
    }
}
fn wildcard_from_mask(mask: u32) -> u32 {
    !mask
}

fn to_binary(octets: [u8; 4]) -> String {
    format!("{:08b} {:08b} {:08b} {:08b}", octets[0], octets[1], octets[2], octets[3])
}

fn binary_octets_with_split(x: u32, prefix: u8) -> String {
    let octets = x.to_be_bytes();
    let mut strs: Vec<String> = vec![];
    let split_bit = (prefix / 8) as usize;
    let split_offset = (prefix % 8) as usize;
    for (i, &o) in octets.iter().enumerate() {
        let s = format!("{:08b}", o);
        if i == split_bit && split_offset != 0 {
            let (a, b) = s.split_at(split_offset);
            strs.push(a.to_string());
            strs.push(b.to_string());
        } else {
            strs.push(s);
        }
    }
    strs.join(" ")
}

fn detect_class_and_priv(addr: Ipv4Addr) -> (String, bool) {
    let o1 = addr.octets()[0];
    let class = match o1 {
        0 => "Address with 0.x (reserved)".to_string(),
        1..=126 => "Class A".to_string(),
        127 => "Loopback".to_string(),
        128..=191 => "Class B".to_string(),
        192..=223 => "Class C".to_string(),
        224..=239 => "Class D (multicast)".to_string(),
        _ => "Class E (reserved)".to_string(),
    };
    let is_private = match addr.octets() {
        [10, _, _, _] => true,
        [172, b, _, _] if (16..=31).contains(&b) => true,
        [192, 168, _, _] => true,
        _ => false,
    };
    (class, is_private)
}

fn main() {
    let args = Args::parse();
    let parts: Vec<&str> = args.input.split('/').collect();
    if parts.len() != 2 {
        eprintln!("{}", "Invalid Format. Use: ipcalc x.x.x.x/(0-32)".red());
        std::process::exit(1);
    }
    let ip = match Ipv4Addr::from_str(parts[0]) {
        Ok(a) => a,
        Err(_) => {
            eprintln!("{}", "Invalid IP".red());
            std::process::exit(1);
        }
    };
    let prefix: u8 = match parts[1].parse() {
        Ok(p) if p <= 32 => p,
        _ => {
            eprintln!("{}", "Invalid prefix (0-32)".red());
            std::process::exit(1);
        }
    };

    let ip_u = ipv4_to_u32(ip);
    let mask = mask_from_prefix(prefix);
    let wildcard = wildcard_from_mask(mask);
    let network = ip_u & mask;
    let broadcast = network | wildcard;

    let hosts_net: i64 = match prefix {
        31 => 2,
        32 => 1,
        _ => {
            let host_bits = 32 - prefix;
            if host_bits == 0 {
                1
            } else {
                ((1u128 << host_bits) - 2) as i64
            }
        }
    };

    let host_min = if prefix >= 31 { network } else { network + 1 };
    let host_max = if prefix >= 31 { broadcast } else { broadcast - 1 };

    let (class, is_private) = detect_class_and_priv(u32_to_ipv4(ip_u));

    let label = |s: &str| s.bright_black();
    let data_blue = |s: &str| s.blue().bold();
    let data_purp = |s: &str| s.magenta().bold();

    let ip_bin = binary_octets_with_split(ip_u, prefix);
    let mask_bin = binary_octets_with_split(mask, prefix);
    let wildcard_bin = to_binary(wildcard.to_be_bytes());
    let network_bin = binary_octets_with_split(network, prefix);
    let hostmin_bin = to_binary(host_min.to_be_bytes());
    let hostmax_bin = to_binary(host_max.to_be_bytes());
    let broadcast_bin = to_binary(broadcast.to_be_bytes());

    println!();
    println!(
        "{}\t{}\t\t{}",
        label("Address:").to_string(),
        data_blue(&format!("{}", u32_to_ipv4(ip_u))).to_string(),
        data_purp(&ip_bin).to_string()
    );
    println!(
        "{}\t{}\t= {}\t{}",
        label("Netmask:").to_string(),
        data_blue(&format!("{}", u32_to_ipv4(mask))).to_string(),
        data_blue(&format!("{}", prefix)).to_string(),
        data_purp(&mask_bin).to_string()
    );
    println!(
        "{}\t{}\t\t{}",
        label("Wildcard:").to_string(),
        data_blue(&format!("{}", u32_to_ipv4(wildcard))).to_string(),
        data_purp(&wildcard_bin).to_string()
    );
    println!();
    println!(
        "{}\t{}\t\t{}",
        label("Network:").to_string(),
        data_blue(&format!("{}/{}", u32_to_ipv4(network), prefix)).to_string(),
        data_purp(&network_bin).to_string()
    );
    println!(
        "{}\t{}\t\t{}",
        label("HostMin:").to_string(),
        data_blue(&format!("{}", u32_to_ipv4(host_min))).to_string(),
        data_purp(&hostmin_bin).to_string()
    );
    println!(
        "{}\t{}\t\t{}",
        label("HostMax:").to_string(),
        data_blue(&format!("{}", u32_to_ipv4(host_max))).to_string(),
        data_purp(&hostmax_bin).to_string()
    );
    println!(
        "{}\t{}\t\t{}",
        label("Broadcast:").to_string(),
        data_blue(&format!("{}", u32_to_ipv4(broadcast))).to_string(),
        data_purp(&broadcast_bin).to_string()
    );


    println!();
    let mut class_info = class;
    if is_private {
        class_info.push_str(", Private Internet");
    }
    println!(
        "{}\t{}\t{}",
        label("Hosts/Net:").to_string(),
        data_blue(&format!("{}", hosts_net)).to_string(),
        data_purp(&format!("{}", class_info)).to_string()
    );
    println!();
}