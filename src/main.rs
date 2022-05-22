use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread::spawn;
use std::time::Instant;

use bitcoin::{
    secp256k1::{Secp256k1, Verification},
    Network,
};
use miniscript::{Descriptor, DescriptorPublicKey, DescriptorTrait};

fn address_prefix_length<T: Verification>(
    desc: &Descriptor<DescriptorPublicKey>,
    secp: &Secp256k1<T>,
) -> usize {
    let address = desc
        .derived_descriptor(&secp, 1)
        .expect("Error: Failed to derive child descriptor")
        .address(Network::Bitcoin)
        .expect("Error: Failed to derive address")
        .to_string();

    if address.chars().nth(0).unwrap().is_numeric() {
        // legacy addresses have 1 character prefix
        return 1;
    } else {
        // segwit addresses have 4 character prefix
        return 4;
    }
}

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        println!("Usage: vanity_descriptors <descriptor> <prefix>");
        return Ok(());
    }

    let desc = Descriptor::<DescriptorPublicKey>::from_str(&args[1])?;
    if !desc.is_deriveable() {
        println!("Error: Not a ranged ('*' inside) descriptor, can't derive addresses");
        return Ok(());
    }

    let secp = Secp256k1::verification_only();
    let prefix = &args[2]; // p2pkh addresses start with one.
    let prefix_len = prefix.len();
    let skipped = address_prefix_length(&desc, &secp);
    let timer = Instant::now();
    let num_threads = num_cpus::get();
    let (sender, receiver) = channel();

    for mut index in 0..num_threads {
        let sender = sender.clone();
        let prefix = prefix.clone();
        let secp = secp.clone();
        let desc = desc.clone();
        spawn(move || loop {
            let address = desc
                .derived_descriptor(&secp, index as u32)
                .expect("Error: Failed to derive child descriptor")
                .address(Network::Bitcoin)
                .expect("Error: Failed to derive address")
                .to_string();

            if address[skipped..skipped + prefix_len] == prefix {
                sender
                    .send(address)
                    .expect("Error: Couldn't send result over channel");
            }

            index += num_threads;
        });
    }

    let address = receiver
        .recv()
        .expect("Error: Couldn't receive result over channel");

    println!("{}", address);
    println!("Took {} seconds", timer.elapsed().as_secs());

    Ok(())
}
