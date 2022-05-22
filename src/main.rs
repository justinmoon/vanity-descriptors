use bitcoin::Network;
use miniscript::{Descriptor, DescriptorPublicKey, DescriptorTrait};
use std::str::FromStr;

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("Usage: vanity_descriptors <descriptor> <prefix>");
        return Ok(());
    }

    let raw_desc = &args[1];
    // p2pkh addresses start with one.
    // eventually we'll need to figure out the prefix to the prefix according to
    // on descriptor-by-descriptor basis
    let prefix = format!("1{}", &args[2]); // p2pkh addresses start with one.
    let desc = Descriptor::<DescriptorPublicKey>::from_str(&raw_desc)?;
    let secp = bitcoin::secp256k1::Secp256k1::verification_only();

    let mut index = 0;
    loop {
        let address = desc
            .derived_descriptor(&secp, index)?
            .address(Network::Bitcoin)?
            .to_string();

        if address.starts_with(&prefix) {
            println!("{}", address);
            return Ok(());
        }

        index += 1;
    }
}