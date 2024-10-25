use core::fmt;
use std::sync::Arc;

pub use jsonrpsee::core::client::Error as JsonRpcError;
use jsonrpsee::{
    core::client::ClientT,
    ws_client::{WsClient, WsClientBuilder},
};
use serde::{Deserialize, Serialize};
use unionlabs::{
    hash::{H160, H256},
    uint::U256,
};

#[derive(macros::Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScrollEip1186ProofResponse {
    #[serde(with = "::serde_utils::hex_string_list")]
    #[debug(wrap = ::serde_utils::fmt::DebugListAsHex)]
    pub account_proof: Vec<Vec<u8>>,
    pub address: H160,
    #[serde(with = "unionlabs::uint::u256_big_endian_hex")]
    pub balance: U256,
    pub poseidon_code_hash: H256,
    pub keccak_code_hash: H256,
    #[serde(with = "unionlabs::uint::u256_big_endian_hex")]
    pub code_size: U256,
    #[serde(with = "::serde_utils::u64_hex")]
    pub nonce: u64,
    pub storage_hash: H256,
    // TODO: Use alloy
    pub storage_proof: Vec<()>,
}

#[derive(Debug, Clone)]
pub struct JsonRpcClient {
    client: Arc<WsClient>,
}

impl JsonRpcClient {
    pub async fn new(url: impl AsRef<str>) -> Result<Self, jsonrpsee::core::client::Error> {
        Ok(Self {
            client: Arc::new(WsClientBuilder::default().build(url).await?),
        })
    }

    pub async fn get_proof(
        &self,
        address: H160,
        location: impl IntoIterator<Item = U256>,
        block: BlockId,
    ) -> Result<ScrollEip1186ProofResponse, jsonrpsee::core::client::Error> {
        self.client
            .request(
                "eth_getProof",
                (
                    address,
                    location
                        .into_iter()
                        .map(|u| (u.to_be_hex_packed()))
                        .collect::<Vec<_>>(),
                    block.to_string(),
                ),
            )
            .await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockId {
    Number(u64),
}

impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockId::Number(n) => write!(f, "0x{n:x}"),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn get_proof_serde() {
//         let json = r#"{"address":"0x58865036d143605698884d7db32c808b4c7afbe7","accountProof":["0x091180bf74ed2741df46f1388e103b4cd966cf7a0d99b699931a9b804d293a016702c3c7e01447462ec2ab4ff681d68c655f77a05959b652f8a0b7085ab29fb5d8","0x092901ec55c896587d6b777b3bd43ad332cc604e779bb1a3296a96ac3aa0d7da5408aeb85ef1c8f6818e2ec551240c2dcdb6234fad21517064bb2d1932fd3c083e","0x090576562158ced393a65715c97b525621b676738bbd20348c93a11e92ce5afcc10d4c9cf69470772f4be5c412d22669616f85358ef9d008b527069e23f12625ae","0x090104691d18dc790c4e12452936f426ed80a4fe0684549f23317e9b0f7ceb29f01134a65e84498c60ddbcee1a5126a4c37093339d56b8dac3a50d1eb154574259","0x09165b3a90946c6b8bd09be831c53c66888b696fdbbce8f0193d0867703e923d0a28c1b826e0746555b4986cca5166823b2eff19c28812464621b4d9809fecc337","0x09197b84fabc86c8193fe032483f3ca05d4ea3ec09e924674850072e47c55c38fc195ad2eb8536d33f8704e26b75d67f4eaebdda42058301747dd5575aa344a0e6","0x091657abc971b7a6f34500a6f0da29c09ef807f81de1b0b10c1ddcb12ed4e9ebaf06c60a9455e35a341408c1c9b8eda419e7c6b130294b591cfcbf28bf075816e0","0x0907168fff32b410a66c0925896e1926e9e645f132d1fae5e4acffb24a18a24da60647dffd375dea5062c63ea95271473eed7f0170d3122c0d11db557af169ed08","0x0928813c9f33e4ca5bbbef0bb7528d2ed0803276e82707a507f7400c7f2a7393081cb2dfc002ae38f4ba90048fe4d12134979991e4300eefc97be6d0f85768f054","0x09100063a66bbdd3e9d8ed9412c8342a2be7229b9c1b395ec47130637e986ae3920f929f60051c3e346852309ccc843615fab1837b9d63f9c6fb86ac16edc128ca","0x0903bc613d80f2ae6ac5c6da39d32a5f3ce50501263ac594bed7f9ea0f65659ef1223991c26a303301c87ae31d108131de445c22347f0d1233a7f4998e7e9cdcc7","0x09028c7cb93c6a0e7e53b2a3613dc051a20cdc2945578030d42607e67c8c2598391a3da127b317adbe9fdc7dc977eb979ad7758d2875f6149b8db305e2b62a9c9b","0x0911ae5db3c9b777119a8f8b056a41e7c35066968748970fd41648515f52222ab11b1648e2c1a06f1ab5e193ec7a89db1605a6feddb5c8166a2bdf13be3e915191","0x09004d19a34b8d1d3fa4e5473996c9aa5304d39e81f12a094cc4f35264a2f5782f1c09272d5939499c5e8e76064f16e4d3134e9030a3be070e66ac48193ebf772e","0x0927440a5417518335087398a56eb1b4ef2dc824e5eae2972089bf4d3d2703b40c2e8fb4c9038f7bcb048d46ba20bd15fd254eb5f7fee7e9ecb906cc805824017d","0x091ac078122d6b1ae91dee8c8bdda032db03ce773f84f12948676ee6de5498c8ab0a29f79e97a3759462f77b6caed8dd261c1ffc8c29e04e35ef7cacfc51290e0c","0x092bf7d02438b6b837ff47f917747fe167946e41f539833b7639821cfa1002b18411a531279c5e92d220010c1a1bcff1d08db187d776694fdda92b28df11b67b30","0x0916c88fcd485becf89df23c1dcbedb08189242d2b59703ad8b081ac17a67fd33a1ab8e0a5a7cad8fbf3a31a032ba68befdf86d853d700ac798296b7aadb4c3c59","0x092bd6004e2fd87475587f3a5671a31f8af8046d5ff4a2b22a45b1abb72178a31617b435c2ad5c4d7113b57adcd1ef189c7f94e64e2a86c2ecb84c5c234456f9a9","0x091fcf53c7203286e69823e96d63b3335aa342b6906450a1cc755795eebcae01b02e4d23c34517269db9650b38a0ba5faf43d2881c8972af9f8a3afa530834be19","0x070a573b6dbcac589b4887fcf94bb93ab302d2fe91adfe996541b0727371034051226b3bd867ffc2b7e00e706d1ec07e8e881f05f29d60b42154afdd09d38b2891","0x0622f272fcd7c1ec4ce66d937b490501f712d850b696a9d7120d257e3e4826f5a70edf245dca5c619fbbfe70c74e65ba6f3cc2b483f1d42df7ecf32308c1262010","0x040eaf8a1641131328a0552a82a12f67ddda464b5e9c1236b9849e7400cc589a4d05080000000000000000000000000000000000000000000000000836000000000000000100000000000000000000000000000000000000000000000000000000000000001b52888cae05bdba27f8470293a7d2bc3b9a9c822d96affe05ef243e0dfd44a0e579336da7a994e47e794baa66e0c2b2f8c0c29b396ab4a7e43ea4ebff261e501396d85a01034c4b49f0a0fe9e62471e917313c1bd6ed756869d461cd62d377d2058865036d143605698884d7db32c808b4c7afbe7000000000000000000000000","0x5448495320495320534f4d45204d4147494320425954455320464f5220534d54206d3172525867503278704449"],"balance":"0x0","poseidonCodeHash":"0x1396d85a01034c4b49f0a0fe9e62471e917313c1bd6ed756869d461cd62d377d","keccakCodeHash":"0xe579336da7a994e47e794baa66e0c2b2f8c0c29b396ab4a7e43ea4ebff261e50","codeSize":"0x836","nonce":"0x1","storageHash":"0x1b52888cae05bdba27f8470293a7d2bc3b9a9c822d96affe05ef243e0dfd44a0","storageProof":[{"key":"0x1","value":"0x0","proof":["0x092ae559c4a5791aa624938167828ea4509d88eaa82114504464c72cbd682e1fd1061c6d68c9639dab7cf8bfb78aadeca93a9bab93dbed21a2c26c92b8877a99e9","0x080b57786fb3f84de0a36e57cb2c13baae5ccffd43be3f75c5590d473128811fc40000000000000000000000000000000000000000000000000000000000000000","0x0618b0b7a56d619daa0810e8137a70bf2dc724490c94c53dc8fd63b5446a881f960d7c59168bf3ce47e73bf8eed28a9e2968d2d08442b3548b6ec3f94d530dfd17","0x042f24f164fb4df482acaa0f1e28c2c15a204fa0fcb918189c55700d2ccb8d06500101000055504f4e4c00000000000000000000000000000000000000000000000000000a200000000000000000000000000000000000000000000000000000000000000004","0x5448495320495320534f4d45204d4147494320425954455320464f5220534d54206d3172525867503278704449"]}]}"#;

//         dbg!(serde_json::from_str::<ScrollEip1186ProofResponse>(json).unwrap());
//     }
// }
