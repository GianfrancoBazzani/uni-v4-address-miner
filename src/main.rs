use clap::Parser;
use ethereum_types::{Address, H256};
use std::str::FromStr;
use std::thread;
use uni_v4_address_miner::{create2_address, mine_salt, score};
use std::sync::{Arc, atomic::AtomicBool};

const UNI_V4_DEPLOYER_ADDRESS: &str = "0x48E516B34A1274f49457b9C6182097796D0498Cb";
const UNI_V4_INIT_CODE_HASH: &str =
    "0x94d114296a5af85c1fd2dc039cdaa32f1ed4b0fe0868f02d888bfc91feb645d9";

#[derive(Parser)]
#[command( about, long_about = None)]
struct Cli {
    miner_address: Option<String>,

    score_threshold: Option<i32>,

    #[arg(short, long, value_name = "NUMBER_OF_THREADS", default_value_t = 8)]
    threads: i32,

    #[arg(short, long, value_name = "DEPLOYER_ADDRESS", default_value = UNI_V4_DEPLOYER_ADDRESS)]
    deployer_address: Option<String>,

    #[arg(short, long, value_name = "INIT_CODE_HASH", default_value = UNI_V4_INIT_CODE_HASH)]
    init_code_hash: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let mut deployer_address: Address = Address::zero();
    let mut init_code_hash: H256 = H256::zero();
    let mut miner_address: Address = Address::zero();
    let mut score_threshold: i32 = 0;
    let threads = cli.threads;

    // Parse the command line arguments
    if let Some(_deployer_address) = cli.deployer_address.as_deref() {
        deployer_address = Address::from_str(_deployer_address).expect("Invalid deployer address");
    }
    if let Some(_init_code_hash) = cli.init_code_hash.as_deref() {
        init_code_hash = H256::from_str(_init_code_hash).expect("Invalid init code hash");
    }
    if let Some(_miner_address) = cli.miner_address.as_deref() {
        miner_address = Address::from_str(_miner_address).expect("Invalid miner address");
    }
    if let Some(_score_threshold) = cli.score_threshold {
        score_threshold = _score_threshold;
    }

    // Validate the command line arguments
    if deployer_address == Address::zero() {
        eprintln!("Invalid deployer address");
        std::process::exit(1);
    }
    if init_code_hash == H256::zero() {
        eprintln!("Invalid initialization code hash");
        std::process::exit(1);
    }
    if miner_address == Address::zero() {
        eprintln!("Error:: Invalid miner address");
        std::process::exit(1);
    }

    println!("Starting address mining...");
    println!("Deployer address: {:?}", &deployer_address);
    println!("Init code hash: {:?}", &init_code_hash);
    println!("Miner address: {:?}", &miner_address);
    println!("Score threshold: {}", score_threshold);
    println!("Number of threads: {}", threads);

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            thread::spawn(move || {
                let salt: H256 = mine_salt(
                    deployer_address,
                    init_code_hash,
                    miner_address,
                    score_threshold,
                );
                let address: Address = create2_address(deployer_address, salt, init_code_hash);
                println!("Found a valid salt!");
                println!("Salt: {:?}", &salt);
                println!("Address: {:?}", &address);
                println!("Address score: {:?}", score(&address));
                std::process::exit(0);
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
}
