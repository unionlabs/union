use alloy_primitives::{Address, B256, hex};
use sha3::{Digest, Keccak256};

const PROXY_INITCODE_HASH: [u8; 32] =
    hex!("21c35dbe1b344a2488cf3321d6ce542f8e9f305544ff09e4993a62319a497c1f");

/// Use to obtain the address using the CREATE3 algorithm from solady's assembly optimized implementation.
///
/// <https://github.com/Vectorized/solady/blob/de9aee59648862bb98affd578248d1e75c7073ad/src/utils/CREATE3.sol#L106>
pub fn predict_deterministic_address(deployer: Address, salt: B256) -> Address {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.push(0xff);
    bytes.extend_from_slice(deployer.as_slice());
    bytes.extend_from_slice(salt.as_slice());
    bytes.extend_from_slice(&PROXY_INITCODE_HASH);

    let hash = Keccak256::digest(&bytes);
    let mut proxy_bytes = [0u8; 20];
    proxy_bytes.copy_from_slice(&hash[12..]);

    // Use proxy address to compute the final contract address.
    // keccak256(rlp(proxy_bytes ++ 0x01)) More here -> https://ethereum.stackexchange.com/a/761/66849
    let mut bytes2: Vec<u8> = Vec::new();
    bytes2.extend_from_slice(&[0xd6, 0x94]);
    // RLP prefix for a list of two items
    bytes2.extend(&proxy_bytes);
    // The proxy address
    bytes2.push(0x01);
    // The nonce of the contract
    let hash2 = Keccak256::digest(&bytes2);

    // resulting hash -> The last 20 bytes (40 characters) of the hash.
    Address::from_slice(&hash2[12..])
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{U256, address, bytes, hex, keccak256};
    use alloy_sol_types::{SolValue, sol};

    sol! {
        #[derive(Debug)]
        struct Params {
            uint256 path;
            uint32 channel;
            bytes token;
        }
    }

    use super::*;
    #[test]
    fn test_known_address() {
        // data for this test case obtain from  https://dashboard.tenderly.co/Kaiserkarel/project/simulator/56ee03c1-60ce-448b-a26d-c1736f9f2d9c?sharedSimulation=true
        let deployer = address!("7b7872fec715c787a1be3f062adedc82b3b06144");

        let token = bytes!("779877A7B0D9E8603169DdbD7836e478b4624789");
        let params = (U256::from(0), 5, token);
        let encoded = params.abi_encode_params();
        let salt: B256 = keccak256(encoded);
        assert_eq!(
            hex::encode(salt),
            "c5e2ad25b6b23cf40cd46a140eac9ce56464d944c28b449d79c57067b90477e8"
        );
        println!("Salt: {}", salt);

        let address = predict_deterministic_address(deployer, salt);
        assert_eq!(
            address,
            address!("d1b482d1b947a96e96c9b76d15de34f7f70a20a1"),
        )
    }
}
