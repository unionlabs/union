use std::{fmt::Debug, path::Path};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[track_caller]
fn ensure_json<T: DeserializeOwned + PartialEq + Debug>(path: impl AsRef<Path>, t: T) {
    let response =
        serde_json::from_str::<JsonRpcResponse<T>>(&std::fs::read_to_string(path).unwrap())
            .unwrap();

    assert_eq!(t, response.result);
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse<T> {
    jsonrpc: String,
    id: i32,
    result: T,
}

mod validators {
    use std::num::NonZeroU64;

    use cometbft_types::{crypto::public_key::PublicKey, types::validator::Validator};
    use hex_literal::hex;
    use unionlabs::bounded::BoundedI64;

    use super::*;

    #[test]
    fn validators() {
        ensure_json(
            "testdata/validators/bartio-6760022.json",
            crate::types::ValidatorsResponse {
                block_height: NonZeroU64::new(6760022).unwrap(),
                validators: vec![
                    Validator {
                        address: hex!("00c3b8dfaf3819df26bfa0917d6fac6b370c9896").into(),
                        pub_key: PublicKey::Bls12_381(
                            "oSdFDUj5WtM+s0NxqOG9v7LuvOFowwQJ8wO+lKk/ZI0GFrRmiKINil+Hua1H+2q/"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 268000000000,
                    },
                    Validator {
                        address: hex!("01fcf3bbe73c154a6ecd8ba4762d439768be88e5").into(),
                        pub_key: PublicKey::Bls12_381(
                            "oe3twcK5Y5XXDie2onwEzHYNWIhI5I3+2/xLLPEWBgUejxBW6AZ9PhsYqxvXgZIK"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 268000000000,
                    },
                    Validator {
                        address: hex!("038c7c4734ad68e3a83487be04c14ab4ec8d494e").into(),
                        pub_key: PublicKey::Bls12_381(
                            "gVlw+QdQ3VsJM7hWoMATEG9tXSVvXq66I3ws21NgIQzmcz9UX+E8GjKINXzaZBzM"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 268000000000,
                    },
                    Validator {
                        address: hex!("071e725312551d5f03e4b24cac63e354b910e8ea").into(),
                        pub_key: PublicKey::Bls12_381(
                            "toUCYnsoa5RQ76NK9VP5lVnttJUcRj9i56Lqj/1IVRcJtAWsj45aTvsBaRQvJqn+"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 268000000000,
                    },
                    Validator {
                        address: hex!("0b61f2d55c4ec7d62c5fdde3b0c81fa0704ffe36").into(),
                        pub_key: PublicKey::Bls12_381(
                            "guxeZ2g8cbMI5U1FO8nE3PVaDY/26ycD+X7IN7dBzvjLVqe6WvEd96HKlddTmWhx"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 236000000000,
                    },
                    Validator {
                        address: hex!("11befa71ff4a8edc1ce2844bf0bd87a1dc1517f7").into(),
                        pub_key: PublicKey::Bls12_381(
                            "oyRxP3lBkuBDk7y6Z+h0T/bN+Joh8sConSOlc+90/ZB8dFVei0voM3tV767p4/Wz"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: -1352000000000,
                    },
                    Validator {
                        address: hex!("12d8cd0b8c92954fe8482e02bca0d9c08cf80732").into(),
                        pub_key: PublicKey::Bls12_381(
                            "icVORRbZGXg51l++UNxcYTwzI4puNFgWipr4W92XsoTwngdibt7nJq5dd8l905NL"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: -1760000000000,
                    },
                    Validator {
                        address: hex!("136ed761c49dcd7685f3d81136e7fb4dfcbbe891").into(),
                        pub_key: PublicKey::Bls12_381(
                            "iXd624+E3CgVRLKx0YzhZIE8MygTJNsj4WHvIy9UcwiKf8F9EUgjUM4fxVXfcFrm"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 826000000000,
                    },
                    Validator {
                        address: hex!("195b9165c04135ded61e25857477c4a3c9e61707").into(),
                        pub_key: PublicKey::Bls12_381(
                            "tsS9+lxydK/Vt/D5TTN88o9SHtBOGUAxJ5gktk4/Edjul+Xf/ny5ivk/mzJ/Ejac"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 236000000000,
                    },
                    Validator {
                        address: hex!("1a8177110622c5316e94e8a4fd6f23b4d1cb70a6").into(),
                        pub_key: PublicKey::Bls12_381(
                            "k25ImsLs9JiB4MY15jm1CbNyIEqMUnxfnhkrXpL3quG14Y6AuABnLmeHzUla7SZ7"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 172000000000,
                    },
                    Validator {
                        address: hex!("1badfd6b19d78a7a4fb78d59daa0ca7745d8e0be").into(),
                        pub_key: PublicKey::Bls12_381(
                            "jDlE6m9EjfIeMT/Qi02pzhG/Awcf53FX54NnSaXIsLG8tNA6vfw4uYsgm3VeYC6q"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 156000000000,
                    },
                    Validator {
                        address: hex!("1d2c30dba0983095d95faebabd57b1ef96cb07fb").into(),
                        pub_key: PublicKey::Bls12_381(
                            "so8zY28l4DU2W0eGN2fXPP62Wuex3an8ssZXQY45iiezr4CxZCifU6dLJE6FiWu1"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 672000000000,
                    },
                    Validator {
                        address: hex!("20d3cc45a56bd901e151a7cf4b79461994cdfd48").into(),
                        pub_key: PublicKey::Bls12_381(
                            "qgDHfGVH6qUTEtKr1LO2+WEuOYAS/CuousRZQUYTAVpGej6x3PusiFBAYA85xC9G"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 156000000000,
                    },
                    Validator {
                        address: hex!("21d6e143df5af2d863d1432f97bdcfc79ece2133").into(),
                        pub_key: PublicKey::Bls12_381(
                            "ksre5fpqDFWobTr2jNUo7H1tDGytfd5EOMbqcDOtQ0ZsKQppvj1dbLQAVSgBJeo2"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 156000000000,
                    },
                    Validator {
                        address: hex!("21f5ce08e5b039e9c78c47864dbb943940d886b8").into(),
                        pub_key: PublicKey::Bls12_381(
                            "iaJSjkhSqt6SLjdlBVB5CrUeKKBoT1bILpwgdYi10u70N4ty3KeRyvByWZZLXBQ0"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 1712000000000,
                    },
                    Validator {
                        address: hex!("24a6ee555cd835c43992008c35dfaab3323bab21").into(),
                        pub_key: PublicKey::Bls12_381(
                            "o2Sc291Lxxlt1d1SF0dwnD5RaVU158jtiTnZ5EW/dkiFa1IT4JtFr6KCuIfZ+fqE"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 1368000000000,
                    },
                    Validator {
                        address: hex!("251955c24eba1a44ef912a10169ce84e8359c704").into(),
                        pub_key: PublicKey::Bls12_381(
                            "rkXu/BM46sJ0KL5ZBktzJdcM0SXMfVQPrzQf7VMxJWzAPpBeVXx+EZv5Wra09hPd"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 124000000000,
                    },
                    Validator {
                        address: hex!("262ce6f34177df645fc9472712e794f8faded0a1").into(),
                        pub_key: PublicKey::Bls12_381(
                            "pAhj7EzQ79Lw3XtcEEfdhN2962EC10nG7b1M2sRHtphzMZOjD5GgN2PIiMrj+vfb"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 92000000000,
                    },
                    Validator {
                        address: hex!("26818358481aa98ec86f9845f7334f570dc705c7").into(),
                        pub_key: PublicKey::Bls12_381(
                            "tuiFAVCdh96w5STJ+oHMxENDEiKXghznUCdtJh0W20tfnKqYVXQIw7xrfWHwLyzB"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: -504000000000,
                    },
                    Validator {
                        address: hex!("293d024c3937120401a1645762fbda0723abf7ce").into(),
                        pub_key: PublicKey::Bls12_381(
                            "gOSvgS3jklwv2xnSfTPkt4gM8nviSbQO82j5wKYzYvMkQ557nQFgvnQ/LIEcN6Ta"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 2440000000000,
                    },
                    Validator {
                        address: hex!("29c82fda5a5e051dfdf62aea9fbae3cce6393b55").into(),
                        pub_key: PublicKey::Bls12_381(
                            "p1iDhnyaOILHOJrh98r8bLCtCCFmrCmzqo/0sO7balPXl/mtSxhfPHZd/6SdKpv8"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 60000000000,
                    },
                    Validator {
                        address: hex!("2ba3b1d9a8bf19b1ff4b14b7e1575757c772123e").into(),
                        pub_key: PublicKey::Bls12_381(
                            "s0GBdTusRqVZCTeb84TdAM/6vVJthLnpme9bSLuBFykJgLonw04BFY9EKF+HGLUI"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: -4000000000,
                    },
                    Validator {
                        address: hex!("2cb94215e7a0c2168dae0805431c263a7b5a1e82").into(),
                        pub_key: PublicKey::Bls12_381(
                            "gxKXZ+GSGNj7tQM2wGl1XplxmJ7PsuRjoOPOa3guRq1EV3T3O6UpQAv6sTB07Y1Q"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: -20000000000,
                    },
                    Validator {
                        address: hex!("2edc26cb83d155007d7deb5a8f52ab7fa79fc23b").into(),
                        pub_key: PublicKey::Bls12_381(
                            "jAHbSDHJ95urWzgAgxXKz2zD/GUqynLkKof9nj1RTPrF+PaLNID7BSvqCmlZlo8m"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: -20000000000,
                    },
                    Validator {
                        address: hex!("2ee9b9e23bc14c4e29886bcb754aa88c50480eef").into(),
                        pub_key: PublicKey::Bls12_381(
                            "hrSbI6qNrKgr80VOwKD0N7BY1SPYTIx7NEhz2wyq+k+DxemaF019rdoto7zYANXU"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: -20000000000,
                    },
                    Validator {
                        address: hex!("301e6fb8adb311fdb3b9a4a3a986efc986c69969").into(),
                        pub_key: PublicKey::Bls12_381(
                            "qjbP9g0/Og/4l8fLl8aFUdyepEbRb0iZ5Sawi7k9Zd6tNL/Iw/hF4lvhpGrKGEGR"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 1952000000000,
                    },
                    Validator {
                        address: hex!("312d25050722ed2db33fd47d46ee15e0ef164672").into(),
                        pub_key: PublicKey::Bls12_381(
                            "q8XlKfyVZzu/JOnUnRNxXqF36zpqwyvg+lpPiJfLLLEp5FtiUTB4BAJa+PKB3R0O"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: -20000000000,
                    },
                    Validator {
                        address: hex!("3306c53d0795cb9e9c160eae3f6bdb50dbecb916").into(),
                        pub_key: PublicKey::Bls12_381(
                            "iU13+rqKJms00CkUcWp8ecNrrho9tuLDkpEWRPGWz0MWLpjXyiVJm53xHhGanE5b"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: -20000000000,
                    },
                    Validator {
                        address: hex!("33c03cd4f7a38fe7c81eed8ed33c789e40718119").into(),
                        pub_key: PublicKey::Bls12_381(
                            "jwsd8m09JX8qdU0k7dE2ZcdlwG9Mt3AszoyRsT3+Jh4ixEgKEgRMQEb2TUcE32Iy"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: -376000000000,
                    },
                    Validator {
                        address: hex!("36a4c6542aa919f0b718594f70fa2bfb253232f5").into(),
                        pub_key: PublicKey::Bls12_381(
                            "pu4pHAk/GC+I0t7z6XrSrkddW3mIErKCLUDviSzcGZs+nM80HvQE4JZTmbOYgI26"
                                .parse()
                                .unwrap(),
                        ),
                        voting_power: BoundedI64::new_const(32000000000).unwrap(),
                        proposer_priority: 496000000000,
                    },
                ],
                count: 30,
                total: 141,
            },
        );
    }
}
