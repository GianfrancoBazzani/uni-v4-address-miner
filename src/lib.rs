use ethereum_types::{Address, H256};
use keccak_hash::keccak;
use rand::Rng;

pub fn mine_salt(
    deployer_address: Address,
    init_code_hash: H256,
    miner_address: Address,
    score_threshold: i32,
) -> H256 {
    let mut h256_bytes: [u8; 32] = [0; 32];
    h256_bytes[..20].copy_from_slice(&miner_address.0);
    loop {
        let random_bytes: [u8; 12] = rand::thread_rng().gen();
        h256_bytes[20..32].copy_from_slice(&random_bytes);
        let salt = H256::from(h256_bytes);
        let address = create2_address(deployer_address, salt, init_code_hash);
        let address_score = score(&address);
        if address_score >= score_threshold {
            return salt;
        }
    }
}

/// Compute the address of a contract created using the `CREATE2` opcode.
/// address = keccak256(0xff + deployer_address_address + salt + keccak256(initialisation_code))[12:]
pub fn create2_address(deployer_address: Address, salt: H256, init_code_hash: H256) -> Address {
    let mut data = [0u8; 85];
    data[0] = 0xff;
    data[1..21].copy_from_slice(&deployer_address.0);
    data[21..53].copy_from_slice(&salt.0);
    data[53..85].copy_from_slice(&init_code_hash.0);

    let hash = keccak(&mut data);
    Address::from_slice(&hash[12..])
}

///  Compute address score
/// https://blog.uniswap.org/uniswap-v4-address-mining-challenge
/// 10 points for each leading 0 nibble
/// 40 points if the address starts with four consecutive 4s
/// 20 points if the first nibble after the four 4s is not a 4
/// 20 points if the last four nibbles are all 4s
/// 1 point for each 4 elsewhere in the address
pub fn score(address: &Address) -> i32 {
    let mut score = 0;

    let mut nibbles = [0u8; 40];
    for i in 0..20 {
        let byte = address.0[i];
        nibbles[2 * i] = byte >> 4;
        nibbles[2 * i + 1] = byte & 0x0f;
    }

    let mut leading_zeroes = 0;
    let mut count_leading_zeroes = true;
    let mut leading_fours = 0;
    let mut count_leading_fours = false;
    for nibble in &nibbles {
        if count_leading_zeroes {
            if *nibble == 0 {
                leading_zeroes += 1;
            } else {
                count_leading_zeroes = false;
                count_leading_fours = true;
            }
        }
        if count_leading_fours {
            if *nibble == 4 {
                leading_fours += 1;
                score += 1;
            } else {
                count_leading_fours = false;
            }
        }
        if !count_leading_fours && !count_leading_zeroes {
            if *nibble == 4 {
                score += 1;
            }
        }
    }

    score += leading_zeroes * 10;
    if leading_fours == 4 {
        score += 60;
    } else if leading_fours > 4 {
        score += 40;
    }
    if address.0[18] == 0x44 && address.0[19] == 0x44 {
        score += 20;
    }

    score
}
