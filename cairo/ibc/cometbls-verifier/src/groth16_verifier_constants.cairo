use garaga::definitions::{E12D, G1Point, G2Line, G2Point, u288, u384};
use garaga::groth16::Groth16VerifyingKey;

pub const N_PUBLIC_INPUTS: usize = 1;

pub const vk: Groth16VerifyingKey = Groth16VerifyingKey {
    alpha_beta_miller_loop_result: E12D {
        w0: u288 {
            limb0: 0xfd1e484e252ab2d0f1d4a806,
            limb1: 0x825bd7c836582db4ceec2c2a,
            limb2: 0x18bde7d6e4a78e0f,
        },
        w1: u288 {
            limb0: 0x842dd46e7aa3164a85df3e9c,
            limb1: 0xe53ea4274538d7258690c5de,
            limb2: 0x180c51d64b51f946,
        },
        w2: u288 {
            limb0: 0xb6e462b37bdbcac66347ec19,
            limb1: 0xe8629545b73bd047b5913684,
            limb2: 0x21d5405ea928037b,
        },
        w3: u288 {
            limb0: 0xcbcaff8f3e180aa97a8cd548,
            limb1: 0x6de164739a5348cfd2b240db,
            limb2: 0x1790cabb96cb1291,
        },
        w4: u288 {
            limb0: 0xb2cb38a2df4ac6fb24c886da,
            limb1: 0x81bc6e541a26719cd2cb9cb1,
            limb2: 0x20b0f804b95fe80e,
        },
        w5: u288 {
            limb0: 0xadcb6b0cc87ac2501755bceb,
            limb1: 0xc1ae30dc7862c30f6d9b437f,
            limb2: 0x187831708c444904,
        },
        w6: u288 {
            limb0: 0x5cd8ac9a4792b2bda4ba6af2,
            limb1: 0x7ec7348cf9de7efb19a3e1a7,
            limb2: 0x25c366bf2b403ca7,
        },
        w7: u288 {
            limb0: 0xaeaa67446ebb49055d7a6ab,
            limb1: 0xb6be494a85670aa60fdb087a,
            limb2: 0x1f790924f0e8dcd,
        },
        w8: u288 {
            limb0: 0x2ff0a2ded46ddba47cf2059a,
            limb1: 0xb884378588fd5cd8b0a94ce2,
            limb2: 0x11c6abc64bd1e825,
        },
        w9: u288 {
            limb0: 0x426657de4198f5ef376ffcb4,
            limb1: 0x6f2c37809ca390b7644f6ff6,
            limb2: 0x1df9469e4fa5dc68,
        },
        w10: u288 {
            limb0: 0xf6267f7a3fcac43e1bdf84e1,
            limb1: 0xd9505a43bcb9836fc8ad0604,
            limb2: 0xd5f0d3b31b9bf39,
        },
        w11: u288 {
            limb0: 0x82407cb81fc2180ea72abb38,
            limb1: 0x36bdd8f4f2d0cd7aa9d232fc,
            limb2: 0xdf4ba0d66518db5,
        },
    },
    gamma_g2: G2Point {
        x0: u384 {
            limb0: 0xf75edadd46debd5cd992f6ed,
            limb1: 0x426a00665e5c4479674322d4,
            limb2: 0x1800deef121f1e76,
            limb3: 0x0,
        },
        x1: u384 {
            limb0: 0x35a9e71297e485b7aef312c2,
            limb1: 0x7260bfb731fb5d25f1aa4933,
            limb2: 0x198e9393920d483a,
            limb3: 0x0,
        },
        y0: u384 {
            limb0: 0x5c2df711ef39c01571827f9d,
            limb1: 0x6da4d435f3b617cdb3af8328,
            limb2: 0x1d9befcd05a5323e,
            limb3: 0x0,
        },
        y1: u384 {
            limb0: 0xf7be3b99e673b13a075a65ec,
            limb1: 0xcbb1ac09187524c7db36395d,
            limb2: 0x275dc4a288d1afb3,
            limb3: 0x0,
        },
    },
    delta_g2: G2Point {
        x0: u384 {
            limb0: 0xe2f3347d679c2cb1867104dc,
            limb1: 0xe4b26eee3932365e6526c8d5,
            limb2: 0x2aca5d2a73f8d34,
            limb3: 0x0,
        },
        x1: u384 {
            limb0: 0x3c06b888d4e03c56d82c97e6,
            limb1: 0x5a26318e5066db729155514e,
            limb2: 0x7b8dbefa90bde07,
            limb3: 0x0,
        },
        y0: u384 {
            limb0: 0x203aff0042d0216f254806f5,
            limb1: 0xd8bad8e79630e19b25e5392a,
            limb2: 0x1696ccafaefe49a5,
            limb3: 0x0,
        },
        y1: u384 {
            limb0: 0x5e4cdc1cb02cc65dc4ba1bf2,
            limb1: 0x98fdd7d1845500c26e497dc3,
            limb2: 0x2edb19cbb2b6ad0c,
            limb3: 0x0,
        },
    },
};

pub const ic: [G1Point; 2] = [
    G1Point {
        x: u384 {
            limb0: 0x8c1a0ff6c8c069ef5ab66b9a,
            limb1: 0x1a20893a08a46c6804493e83,
            limb2: 0x2a81b98e1c997bd0,
            limb3: 0x0,
        },
        y: u384 {
            limb0: 0x7c98576e98c1ad9d6378fb6f,
            limb1: 0x20d4d6a1f157ec94cc7ba620,
            limb2: 0x276938ada8075cec,
            limb3: 0x0,
        },
    },
    G1Point {
        x: u384 {
            limb0: 0x3b95ff9116e2e5df96b36ab7,
            limb1: 0xe35c5ee7fb496efdffda5e5d,
            limb2: 0x179496ce140df89c,
            limb3: 0x0,
        },
        y: u384 {
            limb0: 0xd8cd54e4d5aa3300649f3cfc,
            limb1: 0x3676b7d646e46a5938c8e5f,
            limb2: 0x326e7d44688ce59,
            limb3: 0x0,
        },
    },
];


pub const precomputed_lines: [G2Line; 176] = [
    G2Line {
        r0a0: u288 {
            limb0: 0x1b3d578c32d1af5736582972,
            limb1: 0x204fe74db6b371d37e4615ab,
            limb2: 0xce69bdf84ed6d6d,
        },
        r0a1: u288 {
            limb0: 0xfd262357407c3d96bb3ba710,
            limb1: 0x47d406f500e66ea29c8764b3,
            limb2: 0x1e23d69196b41dbf,
        },
        r1a0: u288 {
            limb0: 0x1ec8ee6f65402483ad127f3a,
            limb1: 0x41d975b678200fce07c48a5e,
            limb2: 0x2cad36e65bbb6f4f,
        },
        r1a1: u288 {
            limb0: 0xcfa9b8144c3ea2ab524386f5,
            limb1: 0xd4fe3a18872139b0287570c3,
            limb2: 0x54c8bc1b50aa258,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x41d51ef7ce7e12535d2722d0,
            limb1: 0x2df47bd1c2e86639635510d9,
            limb2: 0x285ac75e69d788dd,
        },
        r0a1: u288 {
            limb0: 0x12f7163bc1ff66acdcac74ba,
            limb1: 0x6179c729ee1e9634a0c98971,
            limb2: 0x1939cbe5ce88caa2,
        },
        r1a0: u288 {
            limb0: 0x3f776af58b7fb9ac8985a377,
            limb1: 0x3d205cbf0ee7cd453452c5de,
            limb2: 0x1964ef2052dafde1,
        },
        r1a1: u288 {
            limb0: 0x60927fd6961e28bf49372f3d,
            limb1: 0x1f90c947ab310ef99a4c877a,
            limb2: 0x2f0e25819a556728,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x4d347301094edcbfa224d3d5,
            limb1: 0x98005e68cacde68a193b54e6,
            limb2: 0x237db2935c4432bc,
        },
        r0a1: u288 {
            limb0: 0x6b4ba735fba44e801d415637,
            limb1: 0x707c3ec1809ae9bafafa05dd,
            limb2: 0x124077e14a7d826a,
        },
        r1a0: u288 {
            limb0: 0x49a8dc1dd6e067932b6a7e0d,
            limb1: 0x7676d0000961488f8fbce033,
            limb2: 0x3b7178c857630da,
        },
        r1a1: u288 {
            limb0: 0x98c81278efe1e96b86397652,
            limb1: 0xe3520b9dfa601ead6f0bf9cd,
            limb2: 0x2b17c2b12c26fdd0,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xb283a7d2e9789f0a85418bd4,
            limb1: 0xf8af1493203288c14f2e45f5,
            limb2: 0x2a0dd3f377bee86d,
        },
        r0a1: u288 {
            limb0: 0x992f9b668fa9e7c6a2dd7ba9,
            limb1: 0xf422d09ed311ff1858d237c9,
            limb2: 0x27ca8354ad922aa6,
        },
        r1a0: u288 {
            limb0: 0xc9494b44f94a033f28a32fe0,
            limb1: 0x87a0d0750b107a29b7d80bb6,
            limb2: 0x1f4279a8c504f4f3,
        },
        r1a1: u288 {
            limb0: 0xec2574cacb0f994d3654056f,
            limb1: 0x293b4ef2de9e8e864c4cee92,
            limb2: 0x28255bfe26b88c84,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x269cab956da279c37b55da77,
            limb1: 0x8a5bc9e4be98f224342c59b8,
            limb2: 0x8098714775a174c,
        },
        r0a1: u288 {
            limb0: 0x557ab4517a212569fbd0888d,
            limb1: 0x56d67e8c9362c228f6b7e120,
            limb2: 0x172a828d12a8d587,
        },
        r1a0: u288 {
            limb0: 0x28fa5f97b0a0d26a4ef759d0,
            limb1: 0x7b2fe8f772998b18632ea4b3,
            limb2: 0x16ff5f528e56a248,
        },
        r1a1: u288 {
            limb0: 0x7df4ab6a60263578f45ce0a,
            limb1: 0x98bf7c6ed6504963fd34e317,
            limb2: 0x15628f146dc3901,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x6feca4d7e76bad9c6485b4dd,
            limb1: 0x66c6a681748139b8462f708,
            limb2: 0x4b680469faf6dd4,
        },
        r0a1: u288 {
            limb0: 0xfaf5dab07a09d89d816e237a,
            limb1: 0x192925a8f107d8ce52898765,
            limb2: 0x28f75de36d8a16fa,
        },
        r1a0: u288 {
            limb0: 0x3ed4fa010ef8a84bfeb0469d,
            limb1: 0x2a47877120bf98561a4bb396,
            limb2: 0x2f646d76d131430f,
        },
        r1a1: u288 {
            limb0: 0xe8d5131ca4c6315b31480c57,
            limb1: 0xd14c4ca27b520dadd3f86a99,
            limb2: 0x33ecf8ed5374912,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x6c4e24186b96a246f981df8f,
            limb1: 0x1a706f9958e5f2b3e35c261a,
            limb2: 0x1234f85fbc425be2,
        },
        r0a1: u288 {
            limb0: 0x71f75fef0a29f4bbb676ba5d,
            limb1: 0xeb77f32d49eb2eef4a629760,
            limb2: 0x2fcf4f59596004c6,
        },
        r1a0: u288 {
            limb0: 0x3265a03271c6f2a9b3b0e400,
            limb1: 0x518d6de5b00ab4a1438de3a9,
            limb2: 0x3678848c935b15e,
        },
        r1a1: u288 {
            limb0: 0x3a9557549e46e2339714249e,
            limb1: 0x9b5495eed3a0fa561b59082,
            limb2: 0x1d73ac7a11db5fc2,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xa8309a74235454b2dc68dea,
            limb1: 0xcbfbc76ba0351c02afad3522,
            limb2: 0x23bc95ee9d9b7432,
        },
        r0a1: u288 {
            limb0: 0x764ed6d9acb7eec596c3adde,
            limb1: 0x637cf3a2cf0dd6d7021e528b,
            limb2: 0x1d90042189327956,
        },
        r1a0: u288 {
            limb0: 0xf378e71fe5913ba756026a03,
            limb1: 0xe759e1d423da04ca3b40649c,
            limb2: 0x128518b47c6faf76,
        },
        r1a1: u288 {
            limb0: 0x5808cde08cd029ee1c0754be,
            limb1: 0xeb3c8db2d8a28fe2842e8575,
            limb2: 0xbbb4cb730f1c187,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xcbdb8e4161421f33041c4cd0,
            limb1: 0xa11814969284e168319e1c06,
            limb2: 0x15b641064299177c,
        },
        r0a1: u288 {
            limb0: 0x5f4aa50b3e2881127be0be3d,
            limb1: 0x7f09c53b7a2a6fe73107dbb6,
            limb2: 0x46d5f98b3a5be97,
        },
        r1a0: u288 {
            limb0: 0x29dc542b88c6f6c1a8c88769,
            limb1: 0xe048d611f1ed67c01658c7e9,
            limb2: 0x24f4c6aef1273059,
        },
        r1a1: u288 {
            limb0: 0x9959f3c01b1f5c0491d40ae3,
            limb1: 0x67034108e8e5c676ffd71082,
            limb2: 0x190536b710848e10,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xdf2609007513ea7e5a416da8,
            limb1: 0xe390a081696248d73dd106ad,
            limb2: 0x174dc1209fccabc5,
        },
        r0a1: u288 {
            limb0: 0xe362e1bc526b009466e28eb5,
            limb1: 0x188b5a40b5d1309c5face74f,
            limb2: 0x534c43aa39d5089,
        },
        r1a0: u288 {
            limb0: 0x142001f5f1aff79315cc7d8a,
            limb1: 0xe06fad7ec8deb4a60901856c,
            limb2: 0x2d1c7506fb49a5f7,
        },
        r1a1: u288 {
            limb0: 0xe6329eeac7e1a9c1f3bb7329,
            limb1: 0x4dee9658d14b9419aa71e91e,
            limb2: 0x11e03f84b9d48fc6,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd7fc60d08162548eca9cf0e,
            limb1: 0x9b7da435639dd5d02a750b57,
            limb2: 0xaaec6b7bca1abd2,
        },
        r0a1: u288 {
            limb0: 0xad8d6968c25d7f9fc271e50a,
            limb1: 0xb050dcc68961c73a830fb6d,
            limb2: 0x1218db97cc4bb937,
        },
        r1a0: u288 {
            limb0: 0x5b0431ecb4951b29afd3ac2c,
            limb1: 0x7aadbb3b207050572413f0b3,
            limb2: 0xbed072b4c21a487,
        },
        r1a1: u288 {
            limb0: 0xd53e21a4e91ab7cacf3878c,
            limb1: 0x9b6dc1f85c9ea3954ede64fe,
            limb2: 0x2c0dd3404b7a46c9,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x83037ec7a9751db73f9be3b1,
            limb1: 0x1f5357f0f5607469837c23b4,
            limb2: 0x228fbc046ac94781,
        },
        r0a1: u288 {
            limb0: 0xc2ef122547a54846eec3c604,
            limb1: 0x42c3a05fa90bdce13a35bb8d,
            limb2: 0x1a9bc7059ffa16c0,
        },
        r1a0: u288 {
            limb0: 0xa0d1015ac45cb6da70f11fd6,
            limb1: 0x792bcd6aaf975431ab35e4a9,
            limb2: 0x2fc87cfa3c72c077,
        },
        r1a1: u288 {
            limb0: 0x11b368908ac675b556714c08,
            limb1: 0xbbc36003d22d34ffa4f6420d,
            limb2: 0x1a526deea221d053,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x4f9b9a33adc7d0b9d77a4a39,
            limb1: 0x20e8633b7ed87e265667430a,
            limb2: 0x2059fa5c7e77d2ad,
        },
        r0a1: u288 {
            limb0: 0x6825235bfe2daaadf09712d7,
            limb1: 0xe3b3774bc430a2e8a4653d1f,
            limb2: 0x28da2376ea0e8841,
        },
        r1a0: u288 {
            limb0: 0xfad892023206244023ef7567,
            limb1: 0x9ac933e35fdfc49f644dae28,
            limb2: 0x87ce08da7309348,
        },
        r1a1: u288 {
            limb0: 0xf13dae985ac02d7ea8d7f28a,
            limb1: 0xf28134db6a760872e821db75,
            limb2: 0x2901a083df0a5821,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xfff6be1f80f4c384f5910349,
            limb1: 0x35ae764a8808212908e2f210,
            limb2: 0x1812c57ceb0ab978,
        },
        r0a1: u288 {
            limb0: 0x3b795f3c408ba9d679d3b9d4,
            limb1: 0x15322a8571f0274e3df559cc,
            limb2: 0xbccc2ecf828da6,
        },
        r1a0: u288 {
            limb0: 0x7e0f649e2d3b2842b0addd79,
            limb1: 0x147b6504f88d21b2a27392f0,
            limb2: 0x478931f478ee2ac,
        },
        r1a1: u288 {
            limb0: 0x2d69ce9bb94e74503d4eb4dc,
            limb1: 0x6e3c6c5c3ea92c01a11250bd,
            limb2: 0x70810fed62173db,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xc73a10fb81904367ee82f180,
            limb1: 0xc30d0e30bffa8b4c96d5ff10,
            limb2: 0x25af34756438b82f,
        },
        r0a1: u288 {
            limb0: 0xd7ee9b4768e6ffb62e628865,
            limb1: 0xb6d0caf4782e3139a5573645,
            limb2: 0x2e2694f8e8e983ca,
        },
        r1a0: u288 {
            limb0: 0xf636bc6979a001b9b9d64c2a,
            limb1: 0x882042a95b4038d974252f9b,
            limb2: 0x1e35d5988c27b24e,
        },
        r1a1: u288 {
            limb0: 0x77003e6f1a76cfd8105a0a2e,
            limb1: 0xc261d7bc45ae3132687779c9,
            limb2: 0x2ac1b254b9d034f4,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x1cd61f436acb01c54f3b6bd9,
            limb1: 0x3a9c8a2fdc1108a2b2872383,
            limb2: 0x24f14d9c2d89ac0,
        },
        r0a1: u288 {
            limb0: 0x1139d0d260c0d5a461185057,
            limb1: 0x72c68d30345ab2e17c82af97,
            limb2: 0x2e0484b9df45eab5,
        },
        r1a0: u288 {
            limb0: 0xd4643d299f79b4f05ef8adaf,
            limb1: 0x6e4865e1599afe58cf7845ac,
            limb2: 0x27a85d04ae9e9223,
        },
        r1a1: u288 {
            limb0: 0xa09a9c4d4a0e95469322d88b,
            limb1: 0xca05325fcf18d3e7a0b377d3,
            limb2: 0x28784ed0fdc79889,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xac52ec4533ac8f2b5143ac29,
            limb1: 0xaaf5e56345b089e52d91ae7b,
            limb2: 0x1838369b4d45f8dc,
        },
        r0a1: u288 {
            limb0: 0xe4b291b423a9de8d3f2b9185,
            limb1: 0x40fa1387dda550cb0dac4b63,
            limb2: 0x1354ed1c38975de5,
        },
        r1a0: u288 {
            limb0: 0xfdb7655e22a4a31d417544bb,
            limb1: 0xf8bc1d4a5d29c373882b3b5e,
            limb2: 0xac0f5dc6704f80b,
        },
        r1a1: u288 {
            limb0: 0xa848fe8df36077e2efcac7df,
            limb1: 0xe568b3c10a52842292157eef,
            limb2: 0x2fe0633a013d0a1a,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x53ef03adfc91eb3f7e50791c,
            limb1: 0xf03dfb75091d37720f37ac8d,
            limb2: 0x23b07c2e0436ea1a,
        },
        r0a1: u288 {
            limb0: 0x751624329cd93cabd4433c87,
            limb1: 0x60a2a2480b50ffb0a737c610,
            limb2: 0x1cf614f1b5dca06c,
        },
        r1a0: u288 {
            limb0: 0x6e7149091e15bd8f083cf0ce,
            limb1: 0xb142e86dd4069b00e60e0d8e,
            limb2: 0x22e8a554bc1a0a51,
        },
        r1a1: u288 {
            limb0: 0x9bae2388ea22e276925241b8,
            limb1: 0x67a19ce776f59b7f08743077,
            limb2: 0x3027f15ed98cebcf,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xa5cf13056339732ebf0c21c7,
            limb1: 0x64dd99cb8c19137eb17408f8,
            limb2: 0x1d523e5da70f8853,
        },
        r0a1: u288 {
            limb0: 0xf82fb10d0ae2826e3797551a,
            limb1: 0xc0f3a3bff235ca729c642f48,
            limb2: 0x20433aa92a32eff3,
        },
        r1a0: u288 {
            limb0: 0x221d096fc862e3ce649e61c1,
            limb1: 0x11da43f9de9bc329ff4e5177,
            limb2: 0x1438d78ea783d75d,
        },
        r1a1: u288 {
            limb0: 0xcc1e260075b46cc9743bf755,
            limb1: 0x16d62d300c297eeddfbe7247,
            limb2: 0x1278b4f94558e9ef,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xa551a3c7d8df629737fc843b,
            limb1: 0xd61572b7594dbf4a624db771,
            limb2: 0x25fbed7d17c8a87b,
        },
        r0a1: u288 {
            limb0: 0x3f0afb679e5a795033a467ca,
            limb1: 0xbca7c70e212c6482400cca02,
            limb2: 0x23f10de776cd38a8,
        },
        r1a0: u288 {
            limb0: 0x5fefc33c5479d5ccf8691ff,
            limb1: 0x3a546e26e11c4efd4b747e04,
            limb2: 0x18458d883acb91e4,
        },
        r1a1: u288 {
            limb0: 0x1fc292cb99ec56c0dc8459f0,
            limb1: 0x109a63d4731c81b3edb2dc30,
            limb2: 0x1f1c7188a78d256c,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x9db01bf4c7e1630230bf6d51,
            limb1: 0xe48c408c7e06dcddcd7f9bbe,
            limb2: 0x2b0115dc2e28da1a,
        },
        r0a1: u288 {
            limb0: 0x9f1282d2eadfd44bb186a427,
            limb1: 0x155218bdc8761dcf69f5a5f8,
            limb2: 0x1a3668754ed3eed6,
        },
        r1a0: u288 {
            limb0: 0x668a27b2b684ce4eec77f21b,
            limb1: 0x1c33c1a0e5f810f8fa90d264,
            limb2: 0x1652be7872b197ee,
        },
        r1a1: u288 {
            limb0: 0x151d35ebe1f9c9f8589456c0,
            limb1: 0xc75ec2fad426c396a66c6158,
            limb2: 0xc7806c86daae07b,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xca8f6df1607dc44f32560315,
            limb1: 0xbb4f56cb7a7f3831dd890e38,
            limb2: 0x471ae719a2fd354,
        },
        r0a1: u288 {
            limb0: 0xaefef90f27b380fcb65594c8,
            limb1: 0x4503b4a2446da41c491eee1a,
            limb2: 0xb8dc9d433e543a9,
        },
        r1a0: u288 {
            limb0: 0x7ecdbd849ebd01a3e2c34241,
            limb1: 0xf4bfc87a984b0b363b1138c8,
            limb2: 0x1e8d978e2d95d691,
        },
        r1a1: u288 {
            limb0: 0xad0b9048750d659358512e04,
            limb1: 0xdbd0120aa4d695c44472f754,
            limb2: 0x26a2b237bc35320c,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x283e056ecdd9f3fe86602a99,
            limb1: 0x135ff8cdd17ed363d8888f3,
            limb2: 0x2b6eda24b76fb4f9,
        },
        r0a1: u288 {
            limb0: 0xc37d4ed0db5585ccfbdb35d5,
            limb1: 0x34f103457a8cb1a4ffba9b6d,
            limb2: 0x1899dfce5bda496e,
        },
        r1a0: u288 {
            limb0: 0xe8ed80577431de21c6967ef0,
            limb1: 0xaf94f0bb764acf92cefb5b80,
            limb2: 0x13f1f62946c5e16a,
        },
        r1a1: u288 {
            limb0: 0x579f60cbc480f5826a2b7b01,
            limb1: 0xe9e076c01a144ab0c3eb8e77,
            limb2: 0x36f87c4c5dbc7ab,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xaa51cdf5c1514abad7cb8147,
            limb1: 0xadf17ad818aba3185f358169,
            limb2: 0x291ac22ae1b33fc5,
        },
        r0a1: u288 {
            limb0: 0x142516232387820dc776b338,
            limb1: 0x5ce4e694a07e44b414cb4a24,
            limb2: 0x2828aed91d81e31c,
        },
        r1a0: u288 {
            limb0: 0xddd3189116c6f69fd4a10feb,
            limb1: 0x3865ef6037a818988e50cca2,
            limb2: 0x138366b4dfc3e5be,
        },
        r1a1: u288 {
            limb0: 0xbc07bf213a6960e80173df1b,
            limb1: 0x7d4930a1c37ebaf8bf70f693,
            limb2: 0x429dc0e0cc93b86,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xbcfd23d258b5746505b0ecc6,
            limb1: 0x28035578e450a65ef9ba571c,
            limb2: 0x1b64711d78d622a7,
        },
        r0a1: u288 {
            limb0: 0x40f2590c84515817f9aba50b,
            limb1: 0xf826821ff70ba24b674527f9,
            limb2: 0xf7502d2ab2bd262,
        },
        r1a0: u288 {
            limb0: 0x92ca28112074e89dbdcb73f0,
            limb1: 0xaedcd2b46dab8150591e93b5,
            limb2: 0xb9a3c568ac2e7d2,
        },
        r1a1: u288 {
            limb0: 0xc9259ca3378d96ce29ff5eb4,
            limb1: 0xa9f7fa4b5c670635d0751908,
            limb2: 0xda761c6b55bbb0e,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x346547f5ecae6a7198ba099b,
            limb1: 0x47095429abef843a1037d3aa,
            limb2: 0x25f7392b502c30e3,
        },
        r0a1: u288 {
            limb0: 0xf7e87be6fa97fb419d1e1c1d,
            limb1: 0x302db4eb2dc9c2acafb8a888,
            limb2: 0x150cd696c87f168e,
        },
        r1a0: u288 {
            limb0: 0xced0e564669e8bac761af691,
            limb1: 0x680c359009fbe97cdb9d4db,
            limb2: 0x5cd0517de029ae7,
        },
        r1a1: u288 {
            limb0: 0xa0a26f47deb1506f9a9111a2,
            limb1: 0xfd4644932afa36221c0f99ab,
            limb2: 0xb6429c69c6e41e9,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xf3fbeea7d1d1e718d6d6d9f4,
            limb1: 0xb2375ea8dc74378ac3bd5bca,
            limb2: 0x176e1574028ea239,
        },
        r0a1: u288 {
            limb0: 0xea8bd3127692b612793daaf6,
            limb1: 0x258766e1ff143f67fc3c1129,
            limb2: 0xd48da40afa6401e,
        },
        r1a0: u288 {
            limb0: 0x6ae1d12862f20529b09adcf3,
            limb1: 0x5e1bdaf8de341d46831cd4aa,
            limb2: 0x1e32c1a7433c2975,
        },
        r1a1: u288 {
            limb0: 0x12928df11c44787783eac3c0,
            limb1: 0x3194869b2b9f9da626165b8d,
            limb2: 0x17ba57c08c252ec8,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xe9f7c99dd0f145007b44b175,
            limb1: 0x423ede738f1c564aef526eb8,
            limb2: 0x181cf2b3de16a09a,
        },
        r0a1: u288 {
            limb0: 0x7b7783a0406af2c8a297777e,
            limb1: 0xb3fcd3242fff588702cb2189,
            limb2: 0x1e45d717b3914f60,
        },
        r1a0: u288 {
            limb0: 0xd9befa28187846209c386ffa,
            limb1: 0x9500c9b332d49f4a0a1f6eee,
            limb2: 0x1c58447715218966,
        },
        r1a1: u288 {
            limb0: 0x857a86768ab0b24829d19719,
            limb1: 0x44d0c1ef28514f1cdf886600,
            limb2: 0x161a0fdbc590b276,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x9b7988bde003932238349f1f,
            limb1: 0x348cf53b0b2bae83c902625,
            limb2: 0x55be8e443d5fa5e,
        },
        r0a1: u288 {
            limb0: 0xfb8ba9089aca065c1b050ac8,
            limb1: 0x58568a390d30fd585529e7f7,
            limb2: 0x2ff20a1caca8e577,
        },
        r1a0: u288 {
            limb0: 0x7bfa4a4468dbdfa5b11f672a,
            limb1: 0xa5dcad76680c285ce9f85d5d,
            limb2: 0x8a1c914805e3e6c,
        },
        r1a1: u288 {
            limb0: 0x5df7ca1b59f5992330e2ea6d,
            limb1: 0xf005d5e26560e2bda08253f7,
            limb2: 0xc7c5f48b0a35657,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xb1454539cb03d3deb4cf0534,
            limb1: 0x1e4599068e5376e950344ee1,
            limb2: 0x2b46047788cd89d,
        },
        r0a1: u288 {
            limb0: 0x4330d7707783ffe1cd9cd297,
            limb1: 0x25cca3adea2a507ea06a6124,
            limb2: 0x1ec3aaf539543430,
        },
        r1a0: u288 {
            limb0: 0x343c1e2db7b5e53429565b8e,
            limb1: 0x1f6f8f63640138555c942d4a,
            limb2: 0x1a99369613e9f38,
        },
        r1a1: u288 {
            limb0: 0xf040c32c3ac032791c32d847,
            limb1: 0xdf29570202c13a0a88751925,
            limb2: 0x17a008946d6b01e4,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xf76cc840f90658ae9adff1aa,
            limb1: 0xa56e0f7ecb3fad4f3c4f3fb8,
            limb2: 0x74b649254bb5401,
        },
        r0a1: u288 {
            limb0: 0x423981136a2b4a9893621288,
            limb1: 0xb54fc184b5ec68ce72dae9b,
            limb2: 0x5f97cd0e9815133,
        },
        r1a0: u288 {
            limb0: 0xae2415777dafc892d06d334e,
            limb1: 0x5db80b4b86e4a02c9d2603aa,
            limb2: 0x27982ece4cbf361d,
        },
        r1a1: u288 {
            limb0: 0xea81af9a2338bd453919c42,
            limb1: 0xc9184f4b7bf60bc679e4d3e1,
            limb2: 0x44c14c302cbe52e,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x52a26b8d3fa2ac04c39387fe,
            limb1: 0x8afe68e5d7b453354f09a5,
            limb2: 0xe15dcad19bf3299,
        },
        r0a1: u288 {
            limb0: 0xa68f3fa7a73e61bdaed7fde6,
            limb1: 0x134abc0d21f488a58d97a83a,
            limb2: 0x23ebe9f03875a556,
        },
        r1a0: u288 {
            limb0: 0xb8cc1ac61e3e089eb2c031ac,
            limb1: 0x50b286512783dd762d08da6c,
            limb2: 0xc69e08169b54e6c,
        },
        r1a1: u288 {
            limb0: 0x89d79c4be5ca5ccc08603b16,
            limb1: 0x92ecf5ec88ed6913212340d2,
            limb2: 0x2f30464213c7221,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x61b2b6eac1113d60730240fc,
            limb1: 0xc0c2edc5f7c15a6d1b19fede,
            limb2: 0xdd60383235a6749,
        },
        r0a1: u288 {
            limb0: 0x193026823733f5d0befaccf4,
            limb1: 0xbe1091673700f442fbcdae88,
            limb2: 0x6a524cf10bfb3dd,
        },
        r1a0: u288 {
            limb0: 0xe4ef8cbf3d1dce4e37965344,
            limb1: 0x3ea3b2c56da32f387aaa2ff8,
            limb2: 0x2e9772f9ca18cee5,
        },
        r1a1: u288 {
            limb0: 0xc7fd1cb36bb9b07cae1cb691,
            limb1: 0xa95c9eb64d37e406cfb168eb,
            limb2: 0x893b910091beb83,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd0d6bc9a115dd40253c1df1d,
            limb1: 0xb9fe7354ea52fa92c567175f,
            limb2: 0x2ff600dd1107a6a,
        },
        r0a1: u288 {
            limb0: 0x24937ef8d05e99961a61bf89,
            limb1: 0x4f3c55adcd32994b283ae781,
            limb2: 0x1461e515385958b8,
        },
        r1a0: u288 {
            limb0: 0x1430275b2bf74454a44d82f7,
            limb1: 0xa3f835999c9d92c88777577d,
            limb2: 0x15c43fffa2b3724,
        },
        r1a1: u288 {
            limb0: 0xd927d119ee5f884b25b91736,
            limb1: 0x850822688c561518dbafcf79,
            limb2: 0x1ca95e0d73604e94,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xe0cd7d58ff592b11692dce99,
            limb1: 0xa04ed69fffff05278a54dfb1,
            limb2: 0x1496f20487b42ccb,
        },
        r0a1: u288 {
            limb0: 0xf34564878a9954045f3973c,
            limb1: 0x9ae2aeb47627892f4c48ac42,
            limb2: 0x195113107fb23c82,
        },
        r1a0: u288 {
            limb0: 0xdfa700f4d26558b80f27c2d7,
            limb1: 0x22934b370575e7d24a0def94,
            limb2: 0x1177d4b9060a5420,
        },
        r1a1: u288 {
            limb0: 0x840c254e9c40062b7fd63ed2,
            limb1: 0x31c1ffea6d995ac062a05257,
            limb2: 0x5243191f0d2b96e,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xe47d3ad2765ed2d251ba2d10,
            limb1: 0xbef2b2833cec147f1ebacc1b,
            limb2: 0x28a7a65ca2ad7441,
        },
        r0a1: u288 {
            limb0: 0x85c4f5d453bb4def5a23adc,
            limb1: 0xb5458c9ded8464bff9b24f08,
            limb2: 0x6bec470982420d6,
        },
        r1a0: u288 {
            limb0: 0x16622dd7bc1a2047ab948a6c,
            limb1: 0x93a9618a692184f2dc1aa5d7,
            limb2: 0x2d5a9dfd5e002616,
        },
        r1a1: u288 {
            limb0: 0xe25689d91e3827085b5d8d6,
            limb1: 0xa8851f3faad72f9365fc50bf,
            limb2: 0x1c5f063c4781fe20,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xbd979680b791dda1f70418fd,
            limb1: 0xc2c5dba81b26560db798e004,
            limb2: 0x11c899fda9242dfc,
        },
        r0a1: u288 {
            limb0: 0x91f15dcfd0534164519e53d8,
            limb1: 0x29cbd6021ee71d580bea987a,
            limb2: 0xfaff0da3b29bdb0,
        },
        r1a0: u288 {
            limb0: 0x86364c55922bd321484b19de,
            limb1: 0x99f3026c4cf00049639cfacb,
            limb2: 0xf179181e51b5447,
        },
        r1a1: u288 {
            limb0: 0xd9019b3dad8bacc9842937ae,
            limb1: 0x8c003da162bf28298e1571f9,
            limb2: 0x5b14c1c91dbfeed,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xb6d00e0c4357d5b5c47006d1,
            limb1: 0xd4044191d74dc65a14980bc1,
            limb2: 0x2e0db8cec9346e83,
        },
        r0a1: u288 {
            limb0: 0x4a38fc23a8ec13f1c6cfb2c0,
            limb1: 0x1b8ff60d8d303dc57d028ae3,
            limb2: 0x1a082d1ab71c1acd,
        },
        r1a0: u288 {
            limb0: 0x761f12a5e75318d29b7bcc81,
            limb1: 0x35b3231b62edd54fc78c5fd4,
            limb2: 0x1332af26ac4258e7,
        },
        r1a1: u288 {
            limb0: 0xbb60af3e2e389009319ded79,
            limb1: 0x538ea09ee5fe07de96f2dd12,
            limb2: 0x22da9864cf6d9b69,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xa891efa4db1ea9793494938f,
            limb1: 0xbee69bfd1fdeccd6692abfce,
            limb2: 0x156687599d240f38,
        },
        r0a1: u288 {
            limb0: 0x7a2e00f7d3b1785c2ecc8ae1,
            limb1: 0xf8acbe0fecbb1690a89d2656,
            limb2: 0x2016c26ff5b23f60,
        },
        r1a0: u288 {
            limb0: 0xdb2cb48a88c74d9d4d90254c,
            limb1: 0x4e5086b80526958e2e990172,
            limb2: 0x1c3e9d124fa6301,
        },
        r1a1: u288 {
            limb0: 0xf9f7645cc860a38cad7acad8,
            limb1: 0x58a56a7ffd04b84d5fed7046,
            limb2: 0x9a3b3e4180167f3,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x385e49bc583a1b0bf0caf3a0,
            limb1: 0x9d7a7954edaaf59dcae654b,
            limb2: 0x220207bc1564d0bb,
        },
        r0a1: u288 {
            limb0: 0x88f18d7dbfd8ccfdc24b9f28,
            limb1: 0x969d42bb3ddfd21d9d2c2a00,
            limb2: 0xac77d3a0893dd23,
        },
        r1a0: u288 {
            limb0: 0x3c367b1efcbb86baafcb5056,
            limb1: 0xd2c3235c663f62b4143a998,
            limb2: 0x237ada535c8becdb,
        },
        r1a1: u288 {
            limb0: 0x7d92367edad40dff69fe8db5,
            limb1: 0x9220a116ffdb88ccf06c5012,
            limb2: 0x18a919c327e6b722,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x2b6e43455074756916b7f16a,
            limb1: 0xd4dfdb3e42e76122d03ec077,
            limb2: 0x18798c37e07c5a29,
        },
        r0a1: u288 {
            limb0: 0x8c4bcb816bf5cf588850af10,
            limb1: 0x7e971747c61b726abf311b00,
            limb2: 0xc4e987cce2fc033,
        },
        r1a0: u288 {
            limb0: 0xcb951878cf0a989dd7a17a9b,
            limb1: 0x218cb1a85224f8dbfe868bad,
            limb2: 0xfe1fcb45ea6067,
        },
        r1a1: u288 {
            limb0: 0xa7763524ce3a2dc689a837cd,
            limb1: 0xcc16d93a3f0b837903d1637d,
            limb2: 0x1ff723c753a0cb11,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x32ea2e31030f7e4697d90f63,
            limb1: 0x51d2252a1175949788d34eb8,
            limb2: 0xaf48883fae1ef39,
        },
        r0a1: u288 {
            limb0: 0xda6ab95f15dbad80e975f28b,
            limb1: 0xab4304d6b00d3ac4305ee1d3,
            limb2: 0x1362f4bdefac9d7,
        },
        r1a0: u288 {
            limb0: 0x47f4ec81c75165ad75ca48b7,
            limb1: 0xe7d26eb56782c8f135fccd25,
            limb2: 0x2b50573f68d069cd,
        },
        r1a1: u288 {
            limb0: 0xa7eda76323e109db28374d08,
            limb1: 0xfdeba42892e5ad5106036004,
            limb2: 0xa907774416ffe3d,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x1eb615b4e58e6de560bc4788,
            limb1: 0x4177f88f4aec71fad9ab972d,
            limb2: 0x21be7211c554698c,
        },
        r0a1: u288 {
            limb0: 0xc9d206df76ef91da7c0d2549,
            limb1: 0xa70494f598c433dff366320d,
            limb2: 0x30040a6091abcd5b,
        },
        r1a0: u288 {
            limb0: 0x5e036a4060d24b7eade53cc3,
            limb1: 0xc907e90c2ab9d651b48ab974,
            limb2: 0x856683a6365f447,
        },
        r1a1: u288 {
            limb0: 0x9da31b1f4655ec8cbfbdcf29,
            limb1: 0xf2835c83b5193fa8664aa94e,
            limb2: 0x1d72810a5aaf9d1c,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x31284e695f2a62377fd70335,
            limb1: 0x6882f081d3399a70215d0fc8,
            limb2: 0x194ea36ac2fef394,
        },
        r0a1: u288 {
            limb0: 0x601ba338ca87edeaafeec0c4,
            limb1: 0xad0d6d9c2a25ceac6fd9e877,
            limb2: 0x2a69d8d1fc836f11,
        },
        r1a0: u288 {
            limb0: 0x7d4f956eaf4d46549c72bd58,
            limb1: 0x91305a9facccdb3b2fd4cd39,
            limb2: 0xad4b522278ec144,
        },
        r1a1: u288 {
            limb0: 0xb27a7624a9f2cbf11e03e64d,
            limb1: 0xb06e00a0a2f06a6bd99c7cb7,
            limb2: 0x7d308bf900452a8,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x4092ed4e5bbd5116356f576e,
            limb1: 0x5c8938ca207cc477ac82683,
            limb2: 0x1c10f2f98e0a1dc0,
        },
        r0a1: u288 {
            limb0: 0x13727d11f9980a120f8458d8,
            limb1: 0xd43c1ffd8344131917bf1305,
            limb2: 0x37314a2f69b1c29,
        },
        r1a0: u288 {
            limb0: 0xc5c1c26c860eb8ab13a30e0b,
            limb1: 0xa269e798c79b31cf1aed04eb,
            limb2: 0x26e927eaafb12033,
        },
        r1a1: u288 {
            limb0: 0xb5876bed3b2f690c1b0e36df,
            limb1: 0x406138d63514543bf56a217d,
            limb2: 0x719aaeaf796cf74,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x911cde217e01f303c5116b49,
            limb1: 0xa9718bd7d6ef3d51696c8e5a,
            limb2: 0x16853ec6252494b2,
        },
        r0a1: u288 {
            limb0: 0xaeed79053e50e0d6c28ff1be,
            limb1: 0xbec70e716342ce452df014e2,
            limb2: 0x887af05f6c30df8,
        },
        r1a0: u288 {
            limb0: 0x5124c92f9ae3086f2f3d6f1e,
            limb1: 0x3a01dda121c7bdf111950a5f,
            limb2: 0x84540f56ebde2d3,
        },
        r1a1: u288 {
            limb0: 0x8dea171e2ce77dbf2d5d34eb,
            limb1: 0x3891e61e41b05d88a8f0d1da,
            limb2: 0x15ee28dc4aba963a,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd2ba17616fef72508e1a5469,
            limb1: 0xd7dec16d12699c87fd35eed3,
            limb2: 0x195b893be3b8eaf7,
        },
        r0a1: u288 {
            limb0: 0x6dec1515afa975aac35955e9,
            limb1: 0x2eb0396350e6ba0b3b925376,
            limb2: 0x2d9225f5110cbc08,
        },
        r1a0: u288 {
            limb0: 0x3674c208512e6b8adff9ded5,
            limb1: 0x64d240cc4d356d05a91d1a6b,
            limb2: 0xc9c54dbcc0c2dc8,
        },
        r1a1: u288 {
            limb0: 0xdc3916de51ce2cdaab4d2125,
            limb1: 0xc0179c18e2b86aefc8bac7e8,
            limb2: 0x6f0795d4355588e,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x296c3d00d822fbb99bda6205,
            limb1: 0x9945b51e54baca0fbaa92c3a,
            limb2: 0xc5963c4d9ada42f,
        },
        r0a1: u288 {
            limb0: 0x5991dc0b6412e905da8c2c6f,
            limb1: 0xf62fb7489900eb6905c3f5bd,
            limb2: 0x24f135b47e8d2979,
        },
        r1a0: u288 {
            limb0: 0x2c08a9c499dc3750a3897449,
            limb1: 0x951dbbb61e6eb13ae9781606,
            limb2: 0x1335321a292d6d47,
        },
        r1a1: u288 {
            limb0: 0xb1484a35eca604a1519ec2e4,
            limb1: 0xead90a2e92372e9c2ffca4e2,
            limb2: 0x10e321f4be3e6663,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xe9dc4a4481d42008c8276757,
            limb1: 0xb5e45b2fcf716de8ff32b358,
            limb2: 0x260dfae1342e4d80,
        },
        r0a1: u288 {
            limb0: 0xff5c1c8d50493d1800037b0b,
            limb1: 0x765a9e04ed3a06d5dea8a123,
            limb2: 0x2f71580f719a2a02,
        },
        r1a0: u288 {
            limb0: 0x835458619f74a2aec18ed309,
            limb1: 0x867eb8689e5fad56763a0cb0,
            limb2: 0x1becaf3548f3287e,
        },
        r1a1: u288 {
            limb0: 0xba5411b4a031c576e86679c5,
            limb1: 0x9dac7fb7b34f1670091587cb,
            limb2: 0xc7c3ad998270d95,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xaa0b82ed7ec85c7e73604861,
            limb1: 0x576c7f4fd58b1bb3616dbcfd,
            limb2: 0x1338dd390815439,
        },
        r0a1: u288 {
            limb0: 0xcd0b2c4fb8450e0703c7daa7,
            limb1: 0xf43b0ad27c805da74ce6fc76,
            limb2: 0x17382490b5827d40,
        },
        r1a0: u288 {
            limb0: 0x1b9384d7cc6949cc997f43eb,
            limb1: 0xaf9d68f52096313b693df896,
            limb2: 0x149ec4d7e8b9ca4,
        },
        r1a1: u288 {
            limb0: 0x91b2b1d613424d39807cfd20,
            limb1: 0x6ea66d1ad2aadd1a581a560a,
            limb2: 0x14f434c4949fdf78,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x6985e163fa2730a836ac678c,
            limb1: 0x1bd6e2caf5c29f02fad99b40,
            limb2: 0x758737f27911266,
        },
        r0a1: u288 {
            limb0: 0x4d07b724bd2e06333a7ae,
            limb1: 0x679a79ecccb6c67487418b0,
            limb2: 0x44b932705030ac2,
        },
        r1a0: u288 {
            limb0: 0x66fdb0dc4580401a3fa5e820,
            limb1: 0x37649eb77f08da1fae1cc5c0,
            limb2: 0xa5bd2c1e049a08a,
        },
        r1a1: u288 {
            limb0: 0x50acdc4a782a79529e7406be,
            limb1: 0x45d987dc538a83436e63b028,
            limb2: 0x2c59dc52037e0c48,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xc27988541be4ff4ee91df9bc,
            limb1: 0x20f55c725de02fc9d942201f,
            limb2: 0x2bda9d9eb6e547a1,
        },
        r0a1: u288 {
            limb0: 0x978c0c8d8237089d4d03b922,
            limb1: 0x1585fc0d54c17c6ff7e62eaf,
            limb2: 0x4e1822c92dc1c1c,
        },
        r1a0: u288 {
            limb0: 0xbaa1012ecc11bae1fa17d606,
            limb1: 0xce3cad837ba45ab9f64f9bc2,
            limb2: 0xccb1851cfff7977,
        },
        r1a1: u288 {
            limb0: 0xcba4d7833be212bdb92bcf54,
            limb1: 0x7f61c083b5bb9eb600682584,
            limb2: 0x2501e2ea2a14013d,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xe5645311231c86a536a5cd77,
            limb1: 0x5335194af558383d16070751,
            limb2: 0x9d95c905b75462a,
        },
        r0a1: u288 {
            limb0: 0x7a8a3ee304043048d0fdb90a,
            limb1: 0x64404c4ad35f8ad1fc9c2705,
            limb2: 0x1dc253f4a5fb0de7,
        },
        r1a0: u288 {
            limb0: 0xeef5c484567b563e650fb182,
            limb1: 0x80f096b5906ce206dfc06331,
            limb2: 0x22be3ec7b36bf9f0,
        },
        r1a1: u288 {
            limb0: 0x764a484df24a03ba7aeb5d9,
            limb1: 0x5c8aed258279f035f6e2e64d,
            limb2: 0x1c3d1f900ed27f72,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x4de9d6dcb0800a50c5afc20e,
            limb1: 0x97275d20175cb29f4fa42cef,
            limb2: 0x188c20158d6a5ae8,
        },
        r0a1: u288 {
            limb0: 0x3623bb3a8aef13b29b6caf46,
            limb1: 0xf332bbc0518198018f71fb4,
            limb2: 0x216a93a8448b7468,
        },
        r1a0: u288 {
            limb0: 0x10007d156c4e32f91d2a5f01,
            limb1: 0x2199ba6cecc1a438d60364a4,
            limb2: 0x9aa762b425dfbe7,
        },
        r1a1: u288 {
            limb0: 0x5093299d20a2c9f0a7f9a00c,
            limb1: 0xb9cc64ad9ccc48c32efbee68,
            limb2: 0x199e1768c50a6aa0,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x91eb9c72ef7cd95be184249d,
            limb1: 0x21573f48a2adb484fe7f0f9c,
            limb2: 0x2ebac378f05d1056,
        },
        r0a1: u288 {
            limb0: 0x410689107601e9bd1768cbf9,
            limb1: 0x720cabd0a17ddd47ff1ab84b,
            limb2: 0x1d97b695ee090974,
        },
        r1a0: u288 {
            limb0: 0x78ff660deff3831ad4f0b9d0,
            limb1: 0x83c808cc67c2be148ced9b65,
            limb2: 0x1f943ade068b8159,
        },
        r1a1: u288 {
            limb0: 0x893d78534f3cdf6c75eb6704,
            limb1: 0x2708c525f2b2f37633b4ca2e,
            limb2: 0x2181a66524118d86,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x4b1798986b967d575a75df47,
            limb1: 0xeb3e00d8eba4285d382e4b10,
            limb2: 0x25179a093e54ab37,
        },
        r0a1: u288 {
            limb0: 0xec1891df8ce7d8c1cf4f1b56,
            limb1: 0xb5bd9aade8262f22d7b5c945,
            limb2: 0x109327d0273e23c2,
        },
        r1a0: u288 {
            limb0: 0xf9d495623978c09fb8b0c94e,
            limb1: 0x40fb6d6312917e3d71e0f750,
            limb2: 0x182425e04f6e45b7,
        },
        r1a1: u288 {
            limb0: 0x801284392468cea14c288219,
            limb1: 0xc05941f2c52b58d39586d72b,
            limb2: 0x20c14d5068b0f7e7,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xa16969200022abbff00dca34,
            limb1: 0xc50cf06c1d30eabd9268ac13,
            limb2: 0x2ab1e09bc41c1548,
        },
        r0a1: u288 {
            limb0: 0xfad2d482ca1a63f319277269,
            limb1: 0x9bb2fea8c814a950b1e54763,
            limb2: 0x1f8c2a3f6fd82fa2,
        },
        r1a0: u288 {
            limb0: 0x3e1decfebebd29ccfaa9ebd6,
            limb1: 0x60809ab7eafd3951e3225fd4,
            limb2: 0x2edc2f6e461b4326,
        },
        r1a1: u288 {
            limb0: 0xbad6045458dbe58dba4b6ce0,
            limb1: 0x2df12ce3f405b2034d61015b,
            limb2: 0x11fdec7b70c4f7f9,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xac3c2fc0ed7b42ad0a03a994,
            limb1: 0xd08afa55373ca6278ee47a07,
            limb2: 0x23f4697255dab5b,
        },
        r0a1: u288 {
            limb0: 0x5cfb7dfb0fc249ac5328e544,
            limb1: 0xf1cbb69952a4e9fd03c7a078,
            limb2: 0x16b76b63c7047447,
        },
        r1a0: u288 {
            limb0: 0x81d65ba77c82099920aeff1f,
            limb1: 0x8c832ba1695f90da8ba5185b,
            limb2: 0x2967f08d01aae54b,
        },
        r1a1: u288 {
            limb0: 0x5d86976791d15d255f595b28,
            limb1: 0x62b487a95798527a578018e0,
            limb2: 0x2e2c8b1c7c1239cd,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xc46c494353f1e70375c56163,
            limb1: 0x45018b905fefcddb8c9d240c,
            limb2: 0x2f53ab5e410f2d78,
        },
        r0a1: u288 {
            limb0: 0x85bd91294cd036379c586734,
            limb1: 0x77b421470b555736f5cdb2d9,
            limb2: 0x16ba267f96816aa4,
        },
        r1a0: u288 {
            limb0: 0x50d71d2dda1ef824b079dfe5,
            limb1: 0x4cad231cd12226acfddd7b84,
            limb2: 0x1aed29b4b7273b0a,
        },
        r1a1: u288 {
            limb0: 0xc536a1b3971d8b321cb36c10,
            limb1: 0x9225f43211377ea8beb2c7f9,
            limb2: 0x17c9832d6f5c9997,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x393f3bc75584e4a70a195e7f,
            limb1: 0x52fe7eb2c83e288cc23e0e7a,
            limb2: 0x200e9be1e42d1d9d,
        },
        r0a1: u288 {
            limb0: 0x450e0b08d4b92829e6a72877,
            limb1: 0x799bbb55664f97d01e541c4b,
            limb2: 0x133548a6d96df3eb,
        },
        r1a0: u288 {
            limb0: 0xf12abea0d7a60a257cf2d4e9,
            limb1: 0x305c7d6a2f18a834cd2978bc,
            limb2: 0x79aa08aa226d1e3,
        },
        r1a1: u288 {
            limb0: 0xbced16111891c9ad3d66de82,
            limb1: 0x3543d77e974c5c10ef3b7113,
            limb2: 0x25a30230cb5efc2d,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x3eb4870b8dd58faf59997e71,
            limb1: 0x8e63027141e757d0060a7a8d,
            limb2: 0xc50ffb9cc2150e6,
        },
        r0a1: u288 {
            limb0: 0xe7184b0a80b8a30c9a0a3f75,
            limb1: 0xd1487d089c5a37e2d160e77,
            limb2: 0x16b7ec843d2769ad,
        },
        r1a0: u288 {
            limb0: 0x856751590c6d728086ce1a4d,
            limb1: 0xc34f558dda46a0a5bc8065ed,
            limb2: 0x285b9964d26641db,
        },
        r1a1: u288 {
            limb0: 0x127f4908e85d6e82b4382371,
            limb1: 0xc172c4ef66ca6fc9a4b221b6,
            limb2: 0x1ca0a22e5885cef0,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x61df4a38d6439bfafb8fd710,
            limb1: 0xe4b5036678770274635e071d,
            limb2: 0x1844bcf25aed0bb4,
        },
        r0a1: u288 {
            limb0: 0xe6cd12fc9cc3794e1188fb2f,
            limb1: 0xc0ecc2ad1196323143313924,
            limb2: 0x2de8e878ccfad4a,
        },
        r1a0: u288 {
            limb0: 0x43f9fc7cd2dde8ac416950f7,
            limb1: 0x856657cbb2dd42c93cfcd8ae,
            limb2: 0x16b16c6ff9e0bfc7,
        },
        r1a1: u288 {
            limb0: 0xf6bbf8f9be7ca1f553d1ef60,
            limb1: 0xa4bfec7abc5e083b695fe14d,
            limb2: 0x1ef779296f4e0dae,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x972b8823803591c2692761b8,
            limb1: 0xcd0c7e966be2cd6ffecf8c1,
            limb2: 0x29c0cc1bf29aaf16,
        },
        r0a1: u288 {
            limb0: 0x772479f4edba925a969daec9,
            limb1: 0x8345461a9a87f332c33fc6e3,
            limb2: 0x1545ee1f9627356f,
        },
        r1a0: u288 {
            limb0: 0xc9d97a0f95543b71697ac4fe,
            limb1: 0xe8c71fd64255a29bdd7807b3,
            limb2: 0x54bb85ad08b70a1,
        },
        r1a1: u288 {
            limb0: 0x2e2568d4044c34d8994f5aae,
            limb1: 0x4903d9a283f0636fd45ef122,
            limb2: 0x1c6f64d777029dcb,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xde2fa1a4d3635c1bc2569130,
            limb1: 0xa736bd4ddbbd87603881fc4f,
            limb2: 0x1f2f5b36d5a5286b,
        },
        r0a1: u288 {
            limb0: 0x36903b5327ab86eb5a8785d1,
            limb1: 0x4523a8098028629bfc8291c1,
            limb2: 0x265a26635056b5ee,
        },
        r1a0: u288 {
            limb0: 0xc82650717816649a74553cd2,
            limb1: 0xae00f8567d0d1fce6ce64662,
            limb2: 0x2abd3e108387849b,
        },
        r1a1: u288 {
            limb0: 0x3eabf62752f83ca2e71eb2bd,
            limb1: 0x3e176dd48367ff06c6485c9c,
            limb2: 0x166b6678bb700ab8,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x886070142a15a384314dbf7c,
            limb1: 0xa989ca56b797171357615731,
            limb2: 0x1180e991be100fdf,
        },
        r0a1: u288 {
            limb0: 0xc124c126cc8c6254e678e06e,
            limb1: 0x6050652161fb1a8cd5be8116,
            limb2: 0x1b1bd6c00fa11694,
        },
        r1a0: u288 {
            limb0: 0x67a5e4a8694dc2f9fb3030a7,
            limb1: 0xd1a35ba22c1db8126f5f9971,
            limb2: 0x18431c50ee9e252d,
        },
        r1a1: u288 {
            limb0: 0x6a511ae64542a0ea21145799,
            limb1: 0x9e14f5254fed0525d42adac1,
            limb2: 0x1de8f6eabaf70dab,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x7ba6f9aaadc95fa4330ccbdd,
            limb1: 0xd74f41877f29a7ddfa2799c6,
            limb2: 0x2c42ea73ac781405,
        },
        r0a1: u288 {
            limb0: 0xcfe344f0d9a4cb2bd4e64b51,
            limb1: 0xbe643fbc2a22de62fcbc17d0,
            limb2: 0xe35a8094ce6be44,
        },
        r1a0: u288 {
            limb0: 0xd31848f978e76e80c90ed413,
            limb1: 0x8d0e72694f68468c9345c053,
            limb2: 0x2bc292bce74fcb8f,
        },
        r1a1: u288 {
            limb0: 0xe6d22803102f59374a8a8c76,
            limb1: 0x50ee5b2c20ee276c1192c992,
            limb2: 0x54fadbedcaa9d27,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x80ab71de6e753e63103d8420,
            limb1: 0xbc5ee39034e109788b10aca8,
            limb2: 0x6435a1c85b1a7a3,
        },
        r0a1: u288 {
            limb0: 0x22c0079c9afe28b4e4567987,
            limb1: 0x1d826d23bcf54f7f930ed3d5,
            limb2: 0x8e0eaffd0a87a55,
        },
        r1a0: u288 {
            limb0: 0xa78737f888daa5424f5686a0,
            limb1: 0xeca8f6ec0a78ed6b51b00a22,
            limb2: 0x214360c6582c61b6,
        },
        r1a1: u288 {
            limb0: 0x1bdf27fe14a6e6ef31f269bb,
            limb1: 0x7e34096081e43772d6776fde,
            limb2: 0xe9fa5adbc7fb1ac,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd2bb253b4402b8bfeb04afd0,
            limb1: 0x41452349c1b2b174c166f586,
            limb2: 0x113f1f636fa50dca,
        },
        r0a1: u288 {
            limb0: 0xe502a4d1bc50152b635fd76a,
            limb1: 0x9960a3e0cf68f4354ebb8ed1,
            limb2: 0x1eec1e3fe073b96b,
        },
        r1a0: u288 {
            limb0: 0x5bd67cf68f5edfbaefd9eedc,
            limb1: 0xb7641567ef64c788c9de93bc,
            limb2: 0x9ab23893c204c36,
        },
        r1a1: u288 {
            limb0: 0x99d920459a822d4275e62392,
            limb1: 0xf9f5db2a17e54dfbb3f7bd19,
            limb2: 0x24c657e74c49ad28,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xc1f1a8b7a85c29d0295aa7a9,
            limb1: 0x5c23495abcb43d14a311295c,
            limb2: 0x703e7850008109d,
        },
        r0a1: u288 {
            limb0: 0x6a59ed25d634f080b29192c3,
            limb1: 0x6a1a6827f220cfa28357cb03,
            limb2: 0x162c43c078a09945,
        },
        r1a0: u288 {
            limb0: 0xe5b6fb59bf7ff3c2f93a40ee,
            limb1: 0x16f260c74cde5848d1de80e2,
            limb2: 0xc278da41bf5289e,
        },
        r1a1: u288 {
            limb0: 0x9d47ae9bade4e2e23d727155,
            limb1: 0x833cfa917bb5a2b8ce625fcc,
            limb2: 0xa7ffc6c2ff25c65,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd9da1a853c4702fb079586f7,
            limb1: 0x40e9333dd54d66dfa0cf8f08,
            limb2: 0xc26a096993bca67,
        },
        r0a1: u288 {
            limb0: 0x311e5c4803a9e102ddcf7fc,
            limb1: 0xc0ff00ab7eb81bdfb8d81159,
            limb2: 0xeb9b5e44c2449e9,
        },
        r1a0: u288 {
            limb0: 0xb3568f21b69dadd82828682f,
            limb1: 0x33af429f1ac973f7a1c57684,
            limb2: 0x231d8b9d27d34af7,
        },
        r1a1: u288 {
            limb0: 0x17baecb496319c10622ae429,
            limb1: 0x8ec81315c53a6c700b820901,
            limb2: 0x250dc322f9008096,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x89c0b46c512b47e5e9681f1b,
            limb1: 0xc54cd1630d8eaa4c4afa9f51,
            limb2: 0x1cb5e262fc1980db,
        },
        r0a1: u288 {
            limb0: 0x60a0c19597965f3fd91ef451,
            limb1: 0x5c943ace233e96d184dacde2,
            limb2: 0x2c2018e211769000,
        },
        r1a0: u288 {
            limb0: 0xd35c8dfca581574cce908eb,
            limb1: 0xfd5ec08b86014ceefa01213c,
            limb2: 0x1554db1fffe7195e,
        },
        r1a1: u288 {
            limb0: 0xf7bec205baa8901fb946559f,
            limb1: 0xe6cf1a4097e17db952332bf4,
            limb2: 0xdb13e4fb8bc0194,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x2a2dd012965cc1bc40050455,
            limb1: 0x246b04c4700307a9acbf8c6,
            limb2: 0x1b8935f2e69652e7,
        },
        r0a1: u288 {
            limb0: 0x81ae8b5a7837018d2eb51638,
            limb1: 0xc812cc9b596f89f0df39e08c,
            limb2: 0x5234f494f09b103,
        },
        r1a0: u288 {
            limb0: 0x217cb93eb6697625e4d31c88,
            limb1: 0xfbe05c14c37e75541a87d1f5,
            limb2: 0x128cb1fcaf7a2fc6,
        },
        r1a1: u288 {
            limb0: 0x67e7cd950d2fce3b17070ff2,
            limb1: 0x9e8c18e67affd2ace093fe13,
            limb2: 0x1b8ca7fe9be26d37,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x8abceef360b86ee0e162e7ab,
            limb1: 0xc13612b6684009edb89a1390,
            limb2: 0x1b8acae46418876d,
        },
        r0a1: u288 {
            limb0: 0x48353ee0ca8b71b7ac17c637,
            limb1: 0x18870628ddf28c344050310f,
            limb2: 0x28ad74f1bb94f450,
        },
        r1a0: u288 {
            limb0: 0xc1484db1e03fbfd103f05698,
            limb1: 0x17d4fab422c2ebc788a36e3a,
            limb2: 0x2171a8afd24199d7,
        },
        r1a1: u288 {
            limb0: 0xb0816e1663bfa304ad461070,
            limb1: 0x77d2e0939f878a31cb895bb6,
            limb2: 0x16ee28cd883e73f3,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x8897cb1edefc063757f88bb2,
            limb1: 0xff7f3c51b38ea6c2e91df9c5,
            limb2: 0x26244331bad51f8d,
        },
        r0a1: u288 {
            limb0: 0x6e8cdb9f7bf2eb0de38cdf1d,
            limb1: 0x1c0b8e9d5d86cd38865e962a,
            limb2: 0x2e1c33d27536fe5e,
        },
        r1a0: u288 {
            limb0: 0x86e4a3af3361c5224514bc21,
            limb1: 0x7146d95ef1efc3fb0e2992f,
            limb2: 0xf70439190327f7f,
        },
        r1a1: u288 {
            limb0: 0x3091392a348d204da18b807b,
            limb1: 0xb74dfac8d516ca398b4ecbbe,
            limb2: 0x7050566b194aeb5,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xb7817c935d574414f3f28d50,
            limb1: 0xdbf6752ebb5984da629bb127,
            limb2: 0x2c90e0619f25bfd5,
        },
        r0a1: u288 {
            limb0: 0xa062c38d3a767690bfdefb32,
            limb1: 0xb8cba5eac599985113d6180e,
            limb2: 0x640f9c0590a9982,
        },
        r1a0: u288 {
            limb0: 0x1fa259c52d17d54efc044f95,
            limb1: 0xf1ed16bc09cdee13822f9a8e,
            limb2: 0x2437193ef44a47af,
        },
        r1a1: u288 {
            limb0: 0x4899eead6094aaccd061f9d,
            limb1: 0xdd81936ba3dbb813d8aed041,
            limb2: 0x15e24858aded9bed,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xc933b1df292ce0df9d4bfc29,
            limb1: 0x18beb7d0f792f7b922ab42fe,
            limb2: 0x29c67ac0d8acbd55,
        },
        r0a1: u288 {
            limb0: 0x2be6651ceac4ee1ddf634655,
            limb1: 0xbf067c37280ac0f46f6de28e,
            limb2: 0x266230e7780405f4,
        },
        r1a0: u288 {
            limb0: 0x57a72faba40c36e432e67fbb,
            limb1: 0xcde48dbfbf5f2e0c0b1cd99a,
            limb2: 0x1ddf244c7ec0be95,
        },
        r1a1: u288 {
            limb0: 0x2a99d12d33fc7462e299fb5c,
            limb1: 0xc4aa4d53a808e22615840449,
            limb2: 0x1ff2403b6ebb9d14,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd71ecbbe2cce58012bb53589,
            limb1: 0xdeed18f6b4bc6e001b8e5919,
            limb2: 0x2f1c5ace8696577,
        },
        r0a1: u288 {
            limb0: 0x634085cd581d7cd7391dff9f,
            limb1: 0xb9640c66c2482d4bd115bc6a,
            limb2: 0x7e0465f35a7801d,
        },
        r1a0: u288 {
            limb0: 0x89ae196d7ff7d910fee38e80,
            limb1: 0x5cf0b8196a67ba48fae37b22,
            limb2: 0x1fa2ad292abc847b,
        },
        r1a1: u288 {
            limb0: 0x2133bd5544d1573ff5b34be,
            limb1: 0xba78d8af9d1276b453c8f204,
            limb2: 0x27a92c994b231cad,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x2e0a080324ce8b30a6d2acdd,
            limb1: 0x44b9157c4ceac86c1b7a8290,
            limb2: 0x14e26dacb00e379e,
        },
        r0a1: u288 {
            limb0: 0x33e0fab84ba9c5f4fdb85c1b,
            limb1: 0xcc38103db865c7a66928e2da,
            limb2: 0x18c49b1deb28c628,
        },
        r1a0: u288 {
            limb0: 0xd54f0cab37d2adbece044314,
            limb1: 0xbbdbc39b1af422ecccaddb05,
            limb2: 0x27958928b81241d1,
        },
        r1a1: u288 {
            limb0: 0xa5f03f22a6ef9d9103ca4efe,
            limb1: 0x2ad4e09bb01965b95a073d86,
            limb2: 0x289a83d73621a3aa,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x48d0bf090d04bb170053f705,
            limb1: 0x35e0a93cb41538c73c140219,
            limb2: 0xbfc48cde33adcb9,
        },
        r0a1: u288 {
            limb0: 0x3bf56b28bdc615ecef030dbb,
            limb1: 0x63d486620fb3d5f9fd676ffd,
            limb2: 0xe1951133429fb4f,
        },
        r1a0: u288 {
            limb0: 0x1ec9fe26b58d3a3abe76c8ec,
            limb1: 0x743f8dc530de911d8174af81,
            limb2: 0x15381d4e26ad3c70,
        },
        r1a1: u288 {
            limb0: 0x1004010818af58b0d95bf617,
            limb1: 0xe40a00dafb859f0a357136e5,
            limb2: 0x2475eaae4863e42,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x6a566453c961ff95eacbb1e0,
            limb1: 0xec8657663791efa684d51160,
            limb2: 0x1ec82f4576385092,
        },
        r0a1: u288 {
            limb0: 0x9f480abd36775843fb4bbdb,
            limb1: 0xd50f1b2bab447721aa761c46,
            limb2: 0x212078e1f5f81fdd,
        },
        r1a0: u288 {
            limb0: 0x7326c234da30daecc62f0614,
            limb1: 0xb731e5c53a7cc5ba96adf7be,
            limb2: 0x1d8c8ee6f07f1f14,
        },
        r1a1: u288 {
            limb0: 0xe3565d65c8ec718b64ae69b5,
            limb1: 0x663aa23942456733ad12f2ba,
            limb2: 0x8c7c25946ec940a,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xc2fb89fdb91faddc61681699,
            limb1: 0xd749191c6a8f5570a401ae5c,
            limb2: 0x15579c902baa8dc5,
        },
        r0a1: u288 {
            limb0: 0x4748e85bf58bd5b3b00eda29,
            limb1: 0x63919e9d2a3ee95d7278737c,
            limb2: 0x2cf58215232b025f,
        },
        r1a0: u288 {
            limb0: 0x50aa4db4ac8333a64304ac79,
            limb1: 0x5b598abf108a2d43f180d9,
            limb2: 0x28fe26d488ac0d6a,
        },
        r1a1: u288 {
            limb0: 0x4d051cae23425ce5de17fc25,
            limb1: 0x7798d37edee4aabb43c043cb,
            limb2: 0x29145d3e46193961,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x490af1cd8292d6e1f053c8d3,
            limb1: 0x659fb14f37757541860e90e9,
            limb2: 0x2d16b3ff0703a36e,
        },
        r0a1: u288 {
            limb0: 0xf50fc39a3300405dd5c30536,
            limb1: 0x92d7aa235ad3922968ce4269,
            limb2: 0x28877990552247bb,
        },
        r1a0: u288 {
            limb0: 0xb8a5c262566c94f513e0dbcd,
            limb1: 0x2ed487075e74653ddc7411ad,
            limb2: 0x287acda59853c22e,
        },
        r1a1: u288 {
            limb0: 0xebd25fa163066d2f12c597ea,
            limb1: 0xf5ca55f6a52c30c89fae4956,
            limb2: 0x1d3915b24167e6d7,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x326ea41f36ca802a2d460258,
            limb1: 0x2c14bced487edfd659acba49,
            limb2: 0xbbd38cd07a990f1,
        },
        r0a1: u288 {
            limb0: 0x7e7c34dc2a50bc09baaa74b6,
            limb1: 0x4289a223d636f5de335e66af,
            limb2: 0x18b1044202397573,
        },
        r1a0: u288 {
            limb0: 0x816aebfe067408a471dd3474,
            limb1: 0xa183c6764ecd6471c8b3d113,
            limb2: 0x19f59419b27fa3b1,
        },
        r1a1: u288 {
            limb0: 0x609c459c3967a5c90cbfe15e,
            limb1: 0xb56571ad5280bdc1920972be,
            limb2: 0x818b53fa7fcc0b,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x614f76b402bd5c8aaf81a5e4,
            limb1: 0x1cbc0b0544d41342aa6661ef,
            limb2: 0x22aa9c6c52ec9119,
        },
        r0a1: u288 {
            limb0: 0xe781a4b07f3d5cb28e82862e,
            limb1: 0x2e48b3e1af93f7a52931827d,
            limb2: 0x154878971322f6c3,
        },
        r1a0: u288 {
            limb0: 0xbdff276f54487668670b9773,
            limb1: 0x68341beecad29c137c3d2a11,
            limb2: 0x2bffa3e8e8ce84ef,
        },
        r1a1: u288 {
            limb0: 0xfce4b70f1d7c4f422686e696,
            limb1: 0xdafddb6574b909488abc9536,
            limb2: 0x12c160dad27460ff,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x791f77d0ba9e39c446b9b8e6,
            limb1: 0x55bc34e121d4ddfbdf79264a,
            limb2: 0x21c5ccc6aabc0332,
        },
        r0a1: u288 {
            limb0: 0x47ba542375e0c7e303be40da,
            limb1: 0xd34d64f67717d45f67f3b903,
            limb2: 0x83db5636b208e75,
        },
        r1a0: u288 {
            limb0: 0x36e7d6c763a1b7e19e933aaf,
            limb1: 0x6cc782914ab593884d7e795e,
            limb2: 0x1b2c18349b879bdf,
        },
        r1a1: u288 {
            limb0: 0x654335ee6fba5cf9d7f78169,
            limb1: 0x787e1e62edc5cd6d280390a2,
            limb2: 0x26b300e3ba1e0a0d,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x234d82c4ea36abce4d713a26,
            limb1: 0x2a0a9f2dd8b2294284126ece,
            limb2: 0x25ed0b3a8cc4e786,
        },
        r0a1: u288 {
            limb0: 0x1df637395bcd140933da13df,
            limb1: 0xc1f15bc5225dd913b97b9a2c,
            limb2: 0x2e8be1de63504149,
        },
        r1a0: u288 {
            limb0: 0x7b1915e7fe7d5e2495c558a3,
            limb1: 0xcc7ecc4949ec4eed4afb0ad1,
            limb2: 0x28b5c6a4e8955ec7,
        },
        r1a1: u288 {
            limb0: 0xbac0d4a8d66fca59034a478b,
            limb1: 0x2bc0a4567157a66b5adb0f98,
            limb2: 0x25ec33eebd0957c3,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xe1f4fbb49c10070d7f88dd52,
            limb1: 0x4e184803c8e819707ec66c02,
            limb2: 0x2e48aefac027ba82,
        },
        r0a1: u288 {
            limb0: 0xf56a830fd71a73306a997038,
            limb1: 0xb9a22e7eed974c9bba05eef,
            limb2: 0xcf5ddef54439154,
        },
        r1a0: u288 {
            limb0: 0xb30d2784912aae75e3cdee3a,
            limb1: 0x6253d3d39043cbac6bac4c1c,
            limb2: 0x7175d5d3efcf640,
        },
        r1a1: u288 {
            limb0: 0xab5b6a355ccacae11ffdc754,
            limb1: 0xa8b11bffbc723b8efbfb7df5,
            limb2: 0x1e005a1793f9093,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xbcd30c60dcf1ff53da7726f7,
            limb1: 0x4bdd9bb14558efd7cdbda799,
            limb2: 0xdb0ee20c8ee48d5,
        },
        r0a1: u288 {
            limb0: 0xa8d5f59090231aafa1cf548,
            limb1: 0xe599411801fda5cdb256c87f,
            limb2: 0x2f08b4111a9815cf,
        },
        r1a0: u288 {
            limb0: 0x97c83c46d4d21d4b73ec97b9,
            limb1: 0xe38ee07ce6de387a924188b5,
            limb2: 0x4eb7489090a198c,
        },
        r1a1: u288 {
            limb0: 0x4c46bbe91b795c427d858b96,
            limb1: 0x2c10aac0be6a823bd48f986b,
            limb2: 0x2ae645e9b8c8f922,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x8996cb5b3664f6dbac43f1bd,
            limb1: 0xd6a7abdc5fbf7d624f24fa12,
            limb2: 0x119bb68b40f056ea,
        },
        r0a1: u288 {
            limb0: 0x73458e5c6ed35a14253c7367,
            limb1: 0xf1fd9b968c4d76b2c02e478b,
            limb2: 0x646570bfb4df5fa,
        },
        r1a0: u288 {
            limb0: 0xedab34b95a831264a3a24e0a,
            limb1: 0x5b6d4c901ad28f3332717baf,
            limb2: 0xeb8fe90d39a11b2,
        },
        r1a1: u288 {
            limb0: 0xc2273953127edd41f80376e3,
            limb1: 0x5194252b306c548c6f820bb2,
            limb2: 0xcda946d734c359c,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x17ffc77c243484569de7ab,
            limb1: 0xcb651d8fde822cfa15793e09,
            limb2: 0x50486baed8ef00c,
        },
        r0a1: u288 {
            limb0: 0x79efa94c32e5d3a40f576ca9,
            limb1: 0x5cb44254ebd1679868097875,
            limb2: 0x221fa5958245d64,
        },
        r1a0: u288 {
            limb0: 0x2b2c15f6a02689c9a13e5c3d,
            limb1: 0xe577a529b0a3d1f18bb53217,
            limb2: 0x18786e37bec1640a,
        },
        r1a1: u288 {
            limb0: 0x351faa203b7d7e6df6292a3f,
            limb1: 0x33ab12814f947b286d46cbbf,
            limb2: 0x1486823b2420f76a,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x8fe6b3a6b61feae86c5d9d41,
            limb1: 0x851e02024445d86c0a74a41f,
            limb2: 0x4df9bc930416151,
        },
        r0a1: u288 {
            limb0: 0x748f150f617409ea8adc63b6,
            limb1: 0xe087b102833005c6dc7c1d61,
            limb2: 0x1fbcf0267353ebe7,
        },
        r1a0: u288 {
            limb0: 0xf4ab6ad18bb610f5201750f1,
            limb1: 0x965b58e54145a0d66db980f5,
            limb2: 0x256bc45cd6c66a55,
        },
        r1a1: u288 {
            limb0: 0xba8badac309bf9407e7d9477,
            limb1: 0xa3d95e498dd8777e8f1690a5,
            limb2: 0x23b7eac83cfb6e3,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x88e675daf8100c0df083c9f7,
            limb1: 0x8d669f2ae415c16a04a29d25,
            limb2: 0x1a5934840fdf790d,
        },
        r0a1: u288 {
            limb0: 0xa6e740f86c655d88e43618fe,
            limb1: 0x77c2f3ce9cd36a84a2881d8a,
            limb2: 0x8fe333291489ec7,
        },
        r1a0: u288 {
            limb0: 0x54e117e989a1481f2c097a1b,
            limb1: 0xa37ad2b48b81288a6e8e9444,
            limb2: 0x1870543f6dc9c24d,
        },
        r1a1: u288 {
            limb0: 0xeea7118de02d94b41345cdc7,
            limb1: 0xeed66ec2fb8673a7b3090fc1,
            limb2: 0x2a57fb6b39fbf019,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xe7ddccf5826297b5a42fb189,
            limb1: 0x5a79e63d7625ec814c02186d,
            limb2: 0x1893bc098f32290b,
        },
        r0a1: u288 {
            limb0: 0xf63bd2892996db68eb49ee21,
            limb1: 0x97f099a9b80fde68c6349f0,
            limb2: 0x235c7ea529312c94,
        },
        r1a0: u288 {
            limb0: 0x5a76dc281eb5c4efd6f197d1,
            limb1: 0xa77fc703ad1b76832a18a4e2,
            limb2: 0x1393caa98a84663f,
        },
        r1a1: u288 {
            limb0: 0xc9b67f7da6255e1dc8ffbc98,
            limb1: 0x9842dc51df9c15607b1638a,
            limb2: 0x2a3850df66583c77,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x2cce53f7e6e84cec40d7bc50,
            limb1: 0x71a62166ab335917b061d0a7,
            limb2: 0x1a0ef3f11b19041e,
        },
        r0a1: u288 {
            limb0: 0x76774942c3d700548d1d3542,
            limb1: 0x43b139500d3b25f34f077a4b,
            limb2: 0x158bdaf677df4977,
        },
        r1a0: u288 {
            limb0: 0xe9421abb3800baf31bbb70,
            limb1: 0x4315352495e39aea360efdfa,
            limb2: 0x2a05e291988e6424,
        },
        r1a1: u288 {
            limb0: 0x8bf83b2dae74cf4a441991f7,
            limb1: 0xfbda66c31519983ddaf4d2f1,
            limb2: 0x120ea5a147ae318c,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x30044f697543c08f750ce47e,
            limb1: 0xb97d5b6ef2e7364d6ef0f0af,
            limb2: 0x2059a63628ee6aea,
        },
        r0a1: u288 {
            limb0: 0x45d56e64dd1bef1214a0a060,
            limb1: 0x903f3f4583641f985784a39a,
            limb2: 0x18ebbcf067da10d9,
        },
        r1a0: u288 {
            limb0: 0x7ec49e11b208ea2575fd0caa,
            limb1: 0xca50f052bdbd708ababd67a4,
            limb2: 0x25d877fbd5c44580,
        },
        r1a1: u288 {
            limb0: 0x1ac08de4ec487cfcd2e730a,
            limb1: 0x33e1d1e562543f5f19634fef,
            limb2: 0x1fbe3fc7004562b1,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x8360f26cdf98f2d862a8ce09,
            limb1: 0xfb5a6e5e13f6f5595365a492,
            limb2: 0x8bd40b3cd8c697c,
        },
        r0a1: u288 {
            limb0: 0x17f4061dcc7b2de8836868f1,
            limb1: 0x55e82d7d92a0f1d1ff7244af,
            limb2: 0x163986e236367a39,
        },
        r1a0: u288 {
            limb0: 0xd8416787c92dc3c503a112b2,
            limb1: 0xeafffa048703fbb291f62e6b,
            limb2: 0x64dd3aa65616798,
        },
        r1a1: u288 {
            limb0: 0x1eb473dc520d18b74f8117e,
            limb1: 0xb10cc4258b9f6aa0c37768a0,
            limb2: 0x17e79111fc66ed12,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xdfa7b16fb6626caab82aa558,
            limb1: 0xe781765a2200c5f11f842220,
            limb2: 0x19780bbd16496e28,
        },
        r0a1: u288 {
            limb0: 0x5325100af619395d8f66e4da,
            limb1: 0x6200d9cad57bb3699c6e16e5,
            limb2: 0x31ca9f538face82,
        },
        r1a0: u288 {
            limb0: 0xb4d57d2beb632762712dba8d,
            limb1: 0x2486de12f198e9561dc2292d,
            limb2: 0x1612b518975c9103,
        },
        r1a1: u288 {
            limb0: 0x7cf5ebc8843d875b2837f73f,
            limb1: 0xa653aba435f4e3a9c22b5bec,
            limb2: 0x2469b29aebd6c77d,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd3526359e2948273712f16ca,
            limb1: 0x4901ace7a8755a77bcfa50b5,
            limb2: 0x2ecca533dd2f597b,
        },
        r0a1: u288 {
            limb0: 0xfdb79d87411c5d708259e492,
            limb1: 0x9a1074231002e03082eb104e,
            limb2: 0x1665b8177aafc91e,
        },
        r1a0: u288 {
            limb0: 0x52de6662f5354ea3906fdbb4,
            limb1: 0x901dac8f556cb76fa09f26fb,
            limb2: 0xc8915fce8022bc9,
        },
        r1a1: u288 {
            limb0: 0x5ba18ffaae9376af4825bf90,
            limb1: 0xba42aef7eb42bb22e3890a72,
            limb2: 0x106caab3ba31e922,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x38957340b6518009f29c82f6,
            limb1: 0xe60ca9bed1757a98c7d14f85,
            limb2: 0x1e0812b4f625cd4e,
        },
        r0a1: u288 {
            limb0: 0xcb0b8378813b4f670d1e07ea,
            limb1: 0x6ef4444439f0d3284e8b85b2,
            limb2: 0x37e6b907dbf46e8,
        },
        r1a0: u288 {
            limb0: 0xd06aabd53d97c3048d8ec5d3,
            limb1: 0xf459dc6edc558566c26112f1,
            limb2: 0x1b4d71523dea855,
        },
        r1a1: u288 {
            limb0: 0x448c7143f33e6e64d25f32b5,
            limb1: 0xe7509ebfba0ec80a7b691660,
            limb2: 0x2a5f8ded032df077,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x2d91ef5f5d99f6a1b6d0e506,
            limb1: 0x5272c31a078352ad2429bfae,
            limb2: 0x117242ffe4ca450d,
        },
        r0a1: u288 {
            limb0: 0xd9dd78eb3095cb9b535a20ba,
            limb1: 0x432b8ac6e0a7464d3d5622a4,
            limb2: 0x17871728eb41392,
        },
        r1a0: u288 {
            limb0: 0xfdbb8e649216e09483cbaf20,
            limb1: 0x96e7b89be20fcc05da4e28cf,
            limb2: 0x1af4742bc5e2017f,
        },
        r1a1: u288 {
            limb0: 0xe9621cb734c81aad0749c463,
            limb1: 0x23c9bfaf0310fc3eda35e01d,
            limb2: 0x27525cdc24716b8b,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x79af395f2629ce733b2efce9,
            limb1: 0x8cca34f234599aa12bdebfcb,
            limb2: 0x28d6722d6d35803c,
        },
        r0a1: u288 {
            limb0: 0x2068307f9b01e9f8f867daaa,
            limb1: 0x58670ea67819829143321fa0,
            limb2: 0x3c4500563b309f8,
        },
        r1a0: u288 {
            limb0: 0xc164a57aa53171335b918044,
            limb1: 0xb5a297eb045fbcac3f72fbf4,
            limb2: 0x185fcccd620ea326,
        },
        r1a1: u288 {
            limb0: 0xbb7d5cd3d9105439453e461d,
            limb1: 0x2df7fc9e6bb9f2fb04d9ed67,
            limb2: 0xa46195c1de8f217,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x65d9bd71bfe3bea7c3b7c0c7,
            limb1: 0xcd86edc7fd4bd4ea4be26d78,
            limb2: 0x20340c314591383d,
        },
        r0a1: u288 {
            limb0: 0x6a5d370204b7db40fdfe60,
            limb1: 0xc08c6cc15e9e9a9175de379a,
            limb2: 0x7a04c04e4aa6282,
        },
        r1a0: u288 {
            limb0: 0xd41ed91d2bc4ce2837e66dfc,
            limb1: 0x52776ff8576d59c65b60bc98,
            limb2: 0x95144f7cea2edd7,
        },
        r1a1: u288 {
            limb0: 0x4852c1322b2196a4ce35b8ac,
            limb1: 0x384ceb8c04e687d558e0fe69,
            limb2: 0x1c68d352ffa6eb66,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x3c77979c9b844de9dced930f,
            limb1: 0x3e2b66768dba46c33dacd779,
            limb2: 0x1f802bf10ae8986f,
        },
        r0a1: u288 {
            limb0: 0x9a2fb312d552a0d4d0abdf3b,
            limb1: 0x852d9b7426e4b5ed825df7e3,
            limb2: 0xe4a6352d2a401d,
        },
        r1a0: u288 {
            limb0: 0x5c8caf83f041086a2cdb901,
            limb1: 0x7d7b455665c5e9dec651571a,
            limb2: 0x2e0f7c1f172b0d9a,
        },
        r1a1: u288 {
            limb0: 0x6f80521063395b4dd3c8c4da,
            limb1: 0x387cd17337efcafad35445c4,
            limb2: 0x7c3f06283cc7171,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xf97eac879ec0637eddee7c9f,
            limb1: 0xb1e63ea5fd05f253c8a71950,
            limb2: 0xdd8401c5212e669,
        },
        r0a1: u288 {
            limb0: 0xed2a18dc085f629b92af25ac,
            limb1: 0x4d0140c90fcbcaadc7161815,
            limb2: 0x1cb5e0bdbbe3b00f,
        },
        r1a0: u288 {
            limb0: 0xa9a79ac0445016c909591f6d,
            limb1: 0xd47f88dea837eb3f3c866037,
            limb2: 0xa4377cd04abe6,
        },
        r1a1: u288 {
            limb0: 0x3b483245f90c144e3f26ee78,
            limb1: 0xfb765ef375936a4c2c7d2d04,
            limb2: 0x28cfdad039ee1cd5,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x953e817159b012cfe32b742f,
            limb1: 0x59da0c226646b7b02f749fe7,
            limb2: 0xe9b8c67ae18e102,
        },
        r0a1: u288 {
            limb0: 0x918d642210dee23dddbe8b1e,
            limb1: 0xa842cf436200efd11084c4ea,
            limb2: 0x151f6ea2f50206b,
        },
        r1a0: u288 {
            limb0: 0x8edc4a9447cdc47a659daf35,
            limb1: 0x7ba721d643d7dc66d2b3bcff,
            limb2: 0x232f16a78bb1e03f,
        },
        r1a1: u288 {
            limb0: 0x3e00f7619901ede1383abce,
            limb1: 0xb790fe894da399c787f016cd,
            limb2: 0x13f2effc9648049a,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xf5066263f56e67e77cdf595,
            limb1: 0xa092b49770d7c05ca16dff68,
            limb2: 0xef749c7d3eec8d6,
        },
        r0a1: u288 {
            limb0: 0xe612e766752ab9e87866dda0,
            limb1: 0xde5f190958a6d35f37120438,
            limb2: 0xf41cac388d8cdcd,
        },
        r1a0: u288 {
            limb0: 0xe83281bc44dabd1e3d3b18e5,
            limb1: 0xa8097da52cc53c434aa53ab0,
            limb2: 0x150f5891903424cd,
        },
        r1a1: u288 {
            limb0: 0x5c6a4f840f90e514942ceb29,
            limb1: 0x25f98f6575ff3ee4ecec7a3a,
            limb2: 0xe38e46caa6a0273,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x2baaaa6a638bf42241c2279,
            limb1: 0x11818b53924e09edae6e68bc,
            limb2: 0x6c4f6e5d1fc4b43,
        },
        r0a1: u288 {
            limb0: 0xb8f9acbd0b7e172eeb8c2da3,
            limb1: 0x97bb43a94a4743860109d2e6,
            limb2: 0x29bc549372194431,
        },
        r1a0: u288 {
            limb0: 0xd9dfe7b195a3790ccd55af54,
            limb1: 0xd6bda416b358fc4ad10a6a07,
            limb2: 0x297aa17ec0d57206,
        },
        r1a1: u288 {
            limb0: 0xab9c08868626c499720b7388,
            limb1: 0xb1f99dd395b9d607c25d3de7,
            limb2: 0xa700e869f5eede4,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x6f528fdaeb11806a75edb46d,
            limb1: 0x2fe443ba690b14e52f8a5731,
            limb2: 0x109cf082a4cc37ae,
        },
        r0a1: u288 {
            limb0: 0x25eb1f0baf5ff677f55dfd46,
            limb1: 0x5092fa00fdd415957327a326,
            limb2: 0x244a866b9822909b,
        },
        r1a0: u288 {
            limb0: 0xba93be171b042b51453ab312,
            limb1: 0xc0c7126a694888a552b3e920,
            limb2: 0x2f349b69c0e74d6d,
        },
        r1a1: u288 {
            limb0: 0xa12d49ce5d968866092c43dd,
            limb1: 0x85816cdb56b09e1a1fec47e6,
            limb2: 0x27db583f323d638c,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x1a1ae419ff526b3d12cb8fb1,
            limb1: 0xabd153562e21a101fa4361f2,
            limb2: 0x73679c87dc32900,
        },
        r0a1: u288 {
            limb0: 0xf9f3ba5488e998b1beb30298,
            limb1: 0x7be98855ec7127c9119f44c9,
            limb2: 0x2f57a1c6cdbd8e3e,
        },
        r1a0: u288 {
            limb0: 0x1e333ef71495cb83da2dc232,
            limb1: 0xfe0860329f0c4d1a9e44d460,
            limb2: 0x1e06a7b577e8446f,
        },
        r1a1: u288 {
            limb0: 0xb9f773e1f08af1b7781c281e,
            limb1: 0xf48de1f6abf5582cd71dc602,
            limb2: 0x1173881a183279c,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x3853c20773d364db6b4acc23,
            limb1: 0xa652e9409f1760ba87dd5542,
            limb2: 0x16b47175144ea41c,
        },
        r0a1: u288 {
            limb0: 0x2b5e78eda8db65585720c497,
            limb1: 0xe4191025ec397dea13f8f595,
            limb2: 0xb06c3a8b03cbf32,
        },
        r1a0: u288 {
            limb0: 0xd7ccc4d5ccfae8dd6a5012ce,
            limb1: 0x7ef8a2e5fcf503a48fe1a97c,
            limb2: 0x14c38138414acb6e,
        },
        r1a1: u288 {
            limb0: 0xa3f8999194a6452b26a1cc9e,
            limb1: 0x1f1629051a978451abd667de,
            limb2: 0x1279385c73e98e9e,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x9c4f63a47de0055b5b0cfcc0,
            limb1: 0x6d8bbfc4338d8aee21a8a32f,
            limb2: 0x2c4b7f63df8af137,
        },
        r0a1: u288 {
            limb0: 0x674c4fdfb4d1d2c34fc6e34b,
            limb1: 0x6c3b2f3cc1070f329f88f692,
            limb2: 0xe5dc715a6422873,
        },
        r1a0: u288 {
            limb0: 0x66a268358c750fa214a46cad,
            limb1: 0xbfccfda33e033220a61adc66,
            limb2: 0x14fe4481365773dd,
        },
        r1a1: u288 {
            limb0: 0xfaf62e441764b880882b4aa9,
            limb1: 0xa5bc5bc49ec56ed192a7c6db,
            limb2: 0x2c2b5da38914fd02,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x1cb00ad14c503671463e40ee,
            limb1: 0x504bebcfa23720ae262afa1,
            limb2: 0x4fb6b5049818a61,
        },
        r0a1: u288 {
            limb0: 0xfa58a687e0d065efc58957b0,
            limb1: 0xd28acc8d629f2ba369fec065,
            limb2: 0x1306054f9a623fc3,
        },
        r1a0: u288 {
            limb0: 0xb768a2cb99f1f4b1855c90e5,
            limb1: 0x194764b2175e282a70dad941,
            limb2: 0xb04272803542d82,
        },
        r1a1: u288 {
            limb0: 0x8bdfd9a75e2637024f133ba8,
            limb1: 0xd4de09264e6b33ac444d28ff,
            limb2: 0x16434e22a9940286,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xb7160fa0eb214243ca3305fd,
            limb1: 0xef45b93c867cfc092e90293f,
            limb2: 0xa7fd54af803af45,
        },
        r0a1: u288 {
            limb0: 0x107b6183664c7e1b4c01afeb,
            limb1: 0xcdfde08e53601e990b68169f,
            limb2: 0x2e4f007e20e1e32,
        },
        r1a0: u288 {
            limb0: 0x3e42735223864f8ea659e9b3,
            limb1: 0x44c589a0fd38c25dc30e350a,
            limb2: 0xfba3cff44e60475,
        },
        r1a1: u288 {
            limb0: 0x17086bc7272e8cca0a4d9776,
            limb1: 0x50ea9acfb5f7fe3990da8d13,
            limb2: 0x1f3d602fc4df7289,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xa6f033d6adb67122da5f0e3a,
            limb1: 0x9b2d1e85c1ee48dae6c2d7ed,
            limb2: 0x248a8d1bef6a6d59,
        },
        r0a1: u288 {
            limb0: 0xd09fd67801aed9800b7e79c0,
            limb1: 0xb4f54405abb639d5c8ca7822,
            limb2: 0x11aa65b30eaee381,
        },
        r1a0: u288 {
            limb0: 0x937b3b01964430f1a026d89e,
            limb1: 0x565b332a3e1195a9b39eff3b,
            limb2: 0x243399bc8d2871b0,
        },
        r1a1: u288 {
            limb0: 0x4ea2b5ed38ab2fd23453abac,
            limb1: 0x535ace494acfb0b41e557305,
            limb2: 0x243bb76e2cfd8d32,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd65cce5b322f3a8bd27fdb81,
            limb1: 0x1c8777672b3f28542679f9a8,
            limb2: 0x2d4008a677bf268d,
        },
        r0a1: u288 {
            limb0: 0xd4828a6e6e839133742fda61,
            limb1: 0xe9f5d19be939b718c7d1ddfa,
            limb2: 0x20e9cd258a0ac5de,
        },
        r1a0: u288 {
            limb0: 0x96df65b3b52f2865a4d3e13b,
            limb1: 0x65b27fe836681c4e3055005e,
            limb2: 0x1ca3ce987fc6cb5,
        },
        r1a1: u288 {
            limb0: 0x2ad9f3a447d0194a39f71935,
            limb1: 0x6aa059063545a6216032660b,
            limb2: 0x264d009e34a2ba01,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xaaa93307a8561114746292d,
            limb1: 0x4ed1d98ca9fa1e99b56aa221,
            limb2: 0x550a529e3226f88,
        },
        r0a1: u288 {
            limb0: 0xd917e984baf5802ff10990de,
            limb1: 0x1abd57fcfbaf2b0c3aaec6f0,
            limb2: 0x1f7b62bd2142e4bf,
        },
        r1a0: u288 {
            limb0: 0x7eae608d9180f746fb315260,
            limb1: 0xd76a2baf18b8cd4aed56b52b,
            limb2: 0x1fb087d76e931e9a,
        },
        r1a1: u288 {
            limb0: 0x2083c2dbe25a646b5dc90225,
            limb1: 0x295dcd52a8185c2464007a1c,
            limb2: 0x16448a6f98d3aa02,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xcf9919c88e516459ed4ca11b,
            limb1: 0x32b54b1a0b186afc44a89eed,
            limb2: 0x67ce5239a4dfdb7,
        },
        r0a1: u288 {
            limb0: 0x4adac24805caf7e65cd387f1,
            limb1: 0xe84d6c7ba9ed7708e79bb5f6,
            limb2: 0x1d99d69ee5a2b1c5,
        },
        r1a0: u288 {
            limb0: 0xc94895924c686588b7ac1a10,
            limb1: 0xfc04e797c12ad04a7d90e32a,
            limb2: 0x2541a116b970144,
        },
        r1a1: u288 {
            limb0: 0xed955dd4beb54916ca589691,
            limb1: 0x526a76bc3ecefddeaeef0aea,
            limb2: 0xb043d1a485a69ff,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xef65844146537b82db813fe9,
            limb1: 0x85c0067888e1bb8ea0e74c65,
            limb2: 0x25d69f70dc42237b,
        },
        r0a1: u288 {
            limb0: 0x7832b952f58157e163236951,
            limb1: 0x222963ee806502bf311c5b4a,
            limb2: 0x1b7e1f391d68c69a,
        },
        r1a0: u288 {
            limb0: 0x1c525a5d9aa3894d2ed11c4a,
            limb1: 0x58df063d25cd258ade6e52a5,
            limb2: 0x68384894850fd6e,
        },
        r1a1: u288 {
            limb0: 0x235762a4732653d7f4374f1f,
            limb1: 0xdd517414879e454da25ab912,
            limb2: 0x2a6385062c2157f7,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x399662905b852eaca478083,
            limb1: 0x6bb44f4e2d8fa5d69626b245,
            limb2: 0x10532e033b2bb55,
        },
        r0a1: u288 {
            limb0: 0x8c3e01b2dfc92d276ef55c66,
            limb1: 0x66843dee925876a4be4cae5e,
            limb2: 0xd0b7528c94575e5,
        },
        r1a0: u288 {
            limb0: 0x72184631b984d07374dab3cc,
            limb1: 0x7441112833056b3de59622de,
            limb2: 0x159363c7d180e27e,
        },
        r1a1: u288 {
            limb0: 0x4f2d0f243a7edca82ecd564b,
            limb1: 0xa52d2bd76e236a7fa1c6bc2a,
            limb2: 0x2b3eb6499aedfb7f,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xf17435c7552f0c7064612569,
            limb1: 0xf56fc2a65c8adb3c96435eb3,
            limb2: 0xe819f089e2009c3,
        },
        r0a1: u288 {
            limb0: 0xbb48bbe2035a40094d6975ac,
            limb1: 0xdae84d34c34ea7bffd14f3f0,
            limb2: 0x28644760132e0d36,
        },
        r1a0: u288 {
            limb0: 0x9567839881f29dda5daef310,
            limb1: 0xb4bf02a18c95015e359ea7f2,
            limb2: 0xdb632cc13acc807,
        },
        r1a1: u288 {
            limb0: 0xae940144540bd7863380c05,
            limb1: 0x24173048980c84622a159023,
            limb2: 0x1ff7bed8b95d85da,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd5c3433ada49c6828847df29,
            limb1: 0xb5a7cda1eee1da8a34d5823,
            limb2: 0x23e493a4ca0b20ad,
        },
        r0a1: u288 {
            limb0: 0xacd21bae3efe4c72d8cbb64c,
            limb1: 0x9061324c496df4ec0d3d9524,
            limb2: 0x29d364a2b58edbd5,
        },
        r1a0: u288 {
            limb0: 0x9112a1a8e6c0ad05036ce81b,
            limb1: 0xabda752b249b2192d7d7de9e,
            limb2: 0x55cc35e2d7c1ae7,
        },
        r1a1: u288 {
            limb0: 0x16d9007523c5bc329afccf69,
            limb1: 0x6d8146672268bbe1b867629b,
            limb2: 0x8fea9d1bccb7ef3,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x7b18eae79ad773b522591a3e,
            limb1: 0xf967b8da800708b09df3f25,
            limb2: 0x226e2d075cf86c83,
        },
        r0a1: u288 {
            limb0: 0xc12f112d80be3bee384dd127,
            limb1: 0xcad0f83b3e91ed948ecf9e8c,
            limb2: 0x1e4241cd29a81cd7,
        },
        r1a0: u288 {
            limb0: 0x28924ce586391196e39db36b,
            limb1: 0xba7f173043a3347ee188306f,
            limb2: 0x15457732d918185e,
        },
        r1a1: u288 {
            limb0: 0x4355b1be149f1ee91484c21b,
            limb1: 0x669e9dde9a4878fe4a29ae9,
            limb2: 0x2feece55e49575bf,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd5b12c45a6001096b714c92c,
            limb1: 0xe5766f0981a9d5cd30b520c2,
            limb2: 0x23da8ba2397ed80a,
        },
        r0a1: u288 {
            limb0: 0x208e08bdcf451cd7da0570c8,
            limb1: 0x519cfeace61d21e44a9107a6,
            limb2: 0x17af81ac9650f7d2,
        },
        r1a0: u288 {
            limb0: 0xeb1a372cd1acd9a2cc0bb4fd,
            limb1: 0x18494480b75392ebe4f32df4,
            limb2: 0x14a418fc00e60b5a,
        },
        r1a1: u288 {
            limb0: 0x4cc96f63b4c2288ac78b90ae,
            limb1: 0xa2c1168bb545355cdbe1d86c,
            limb2: 0x2c36c3e6aa470863,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x42dc40ecdb76d58aeb725220,
            limb1: 0x1f27d141e96e1f33fe8cf27f,
            limb2: 0x114f6f12388a2c43,
        },
        r0a1: u288 {
            limb0: 0x52ee83be2cae450f3c279e0f,
            limb1: 0x7601c5c4dfc6b6e57d975ba1,
            limb2: 0x2439c056f5b328f5,
        },
        r1a0: u288 {
            limb0: 0x518e7c8aa5964e5fad566bdb,
            limb1: 0x2e31c2f3b96451c098c42d5f,
            limb2: 0x2b297bc1df4ee38c,
        },
        r1a1: u288 {
            limb0: 0x9824e994964c9a66357f04f2,
            limb1: 0x33913442895b7de78d6ffa0a,
            limb2: 0x106c22067c33eea2,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd4311d79fb9d72480d75f4cc,
            limb1: 0x64d549aa7482c86635bf1c88,
            limb2: 0x2bd603b1e0b09b9c,
        },
        r0a1: u288 {
            limb0: 0xc4f21262387dcb5badd7f0f8,
            limb1: 0xab18adc624f1a00e2c1e5c95,
            limb2: 0x264704574b35c2a,
        },
        r1a0: u288 {
            limb0: 0xa3149bdb518b59a0f0fc16ec,
            limb1: 0xd70e764e74d601155b7e85c9,
            limb2: 0x14f31515fb637f7a,
        },
        r1a1: u288 {
            limb0: 0x8377156b4aff61f9a86c9769,
            limb1: 0x68c313eef38d7c78ec0f4b9e,
            limb2: 0x83b5c5d8cc1361b,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xdb82079a8cf65b9420ec84f9,
            limb1: 0x20a432030511989660dd963a,
            limb2: 0x29e09ca4e15b955c,
        },
        r0a1: u288 {
            limb0: 0xc60b2274b193f8d9087ed045,
            limb1: 0x9f7cb26a694880a0af663ba5,
            limb2: 0x2e5da216038d7cb2,
        },
        r1a0: u288 {
            limb0: 0xd83ea43af8a11dff5ab94abb,
            limb1: 0x4292c41d3e43f7e662401ba8,
            limb2: 0x68dca455e98d0ab,
        },
        r1a1: u288 {
            limb0: 0xba83d61fad7f72e3fd7ce466,
            limb1: 0xfbde0c07e46d05381749ab95,
            limb2: 0xd7c62956e6aa887,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xab525c01c84130b181cdda71,
            limb1: 0xf9bc94febf8c89abb247df3d,
            limb2: 0x292f00eed02db8e6,
        },
        r0a1: u288 {
            limb0: 0x6526ccf650f70133d48af395,
            limb1: 0xec8e9754923c619ad2b5aed6,
            limb2: 0xf2951bb57bd7eab,
        },
        r1a0: u288 {
            limb0: 0x7aba815a65f9e7a3609c3755,
            limb1: 0xcd55486bcc307144d726cc6b,
            limb2: 0x6188e46bff2b128,
        },
        r1a1: u288 {
            limb0: 0x51f1ab59658532383c7ce49b,
            limb1: 0xbe24ede4a9f5256043afec1,
            limb2: 0xa957adb2e6f8d03,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xe4170ff87e6e102d5f583f8,
            limb1: 0x23779eadf73da457ded599a1,
            limb2: 0x81bd5569b688efc,
        },
        r0a1: u288 {
            limb0: 0x9eb1cbdfadccfb466e94f96a,
            limb1: 0xdd5ef8b43e39a766efe7808,
            limb2: 0x15af220853c0dcc,
        },
        r1a0: u288 {
            limb0: 0xfbc272cd85d47cb44f7de1f0,
            limb1: 0x2f67969c3565a0a8c03e41d5,
            limb2: 0x549ca1d1fc930ae,
        },
        r1a1: u288 {
            limb0: 0xc3bcce79f4291d8fe4e79782,
            limb1: 0xb6234250820412e672db74ed,
            limb2: 0x1dcb6a58c655ed3f,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x33e3b557be8df2315679cc11,
            limb1: 0x64f8b96fd0269d23bd4bc523,
            limb2: 0x20209760e8c5ecea,
        },
        r0a1: u288 {
            limb0: 0xc8cf99e7126939fee82b6ecb,
            limb1: 0x414f1b6ac90c453af37b6843,
            limb2: 0x1e4620ee839a7994,
        },
        r1a0: u288 {
            limb0: 0x1270d7b81cfeb2372c471237,
            limb1: 0x4871e3be0a85e1fc77c8fe77,
            limb2: 0x46f52c38560cf4a,
        },
        r1a1: u288 {
            limb0: 0x913f2081dabee116fd93aa23,
            limb1: 0x48b4cf30a7d558b5218462c,
            limb2: 0x65537323e9db939,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x866ecceec6ec8c4dcf4fb3bc,
            limb1: 0xf39cf121e0f289960ff9f8da,
            limb2: 0x23afdf0818a34e46,
        },
        r0a1: u288 {
            limb0: 0x727731c5200899dff2c48485,
            limb1: 0xb20b8b5f267c0d7f90d29875,
            limb2: 0x21a1c416b7e5c9f9,
        },
        r1a0: u288 {
            limb0: 0x1983e704304c5e35d72c251a,
            limb1: 0xd5a2afe55aea19c8756013b6,
            limb2: 0x206363cef6fb8069,
        },
        r1a1: u288 {
            limb0: 0x846193b80c13a771bb692092,
            limb1: 0xe13c4641be8f552ab538f8d9,
            limb2: 0x13d43da02ae691c7,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xbe8edae92175f36b7f0a7407,
            limb1: 0xa21441908b14712fe88e7552,
            limb2: 0x795cc16a26f851,
        },
        r0a1: u288 {
            limb0: 0x2cba1ab97b78f0ac15b83ce4,
            limb1: 0xc9f39187f6558ec3b46e8a5f,
            limb2: 0x15724757f9c04e65,
        },
        r1a0: u288 {
            limb0: 0x4b82ae7fb2c9b44825cb9a60,
            limb1: 0x9824fabc9c6278957ced2ca4,
            limb2: 0x23c5ba32e554a25e,
        },
        r1a1: u288 {
            limb0: 0x86e6af58c29581fe02cfb96a,
            limb1: 0x6268223f4fed48568de673d8,
            limb2: 0x1bf18e49d37c5fe7,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x6c47159b5e559835795b44fc,
            limb1: 0x74395de0880c53c7698147d5,
            limb2: 0x8ea3a5e282085d5,
        },
        r0a1: u288 {
            limb0: 0x5c3dc31c937be42d6b7580a9,
            limb1: 0x1c5b55a0f436e09de73d773,
            limb2: 0x335539749d7243d,
        },
        r1a0: u288 {
            limb0: 0xbd2cdd557ab6cd9a1e85709f,
            limb1: 0x8ee6ffd90d993233e0a6ce0b,
            limb2: 0x26a4094dd079fa5e,
        },
        r1a1: u288 {
            limb0: 0x7ae2733215d1be6a19f32a23,
            limb1: 0xa79a181bad5bd2724179420e,
            limb2: 0x709e9bc2a01d130,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x5d2551295b4eb00eaa9c8ae9,
            limb1: 0x80f5cb790b283f7f7301286e,
            limb2: 0x214cd2fb310046bc,
        },
        r0a1: u288 {
            limb0: 0xe0ca10c74b1b8b66246f8329,
            limb1: 0x492cf898b9901f1241fbe280,
            limb2: 0xa3bcd2c7b276715,
        },
        r1a0: u288 {
            limb0: 0xc2413e036d624bd25dbdb2ad,
            limb1: 0xb1f854b8a76da664a9501e6e,
            limb2: 0x1962a3b6bc270362,
        },
        r1a1: u288 {
            limb0: 0xcab80b272e590f58b0f455f2,
            limb1: 0xfa5683f52735835af6676ef8,
            limb2: 0x1baf7035ba743947,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x14ad2e2a718a8b98c439aab2,
            limb1: 0x32a18d309d6f1bb4c45e3ab2,
            limb2: 0x29144cf60028f436,
        },
        r0a1: u288 {
            limb0: 0xae05d6aadee98a93f3faeb0e,
            limb1: 0x7e71c1f74a7585a38612e68b,
            limb2: 0x4e036cbb58ac6e9,
        },
        r1a0: u288 {
            limb0: 0xbf4ed587eb4d42923d68cccc,
            limb1: 0x61b8deff82b6f719f1a38bf3,
            limb2: 0x2da087fc860c000c,
        },
        r1a1: u288 {
            limb0: 0x85bc9715fe53bb174cc0c30,
            limb1: 0xfd9bb28a96c81a7efc25e035,
            limb2: 0x147057da0326d1c8,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd11a4ce94a5c008e2f19c6b3,
            limb1: 0x754ff3a3ea205ebb4d79281f,
            limb2: 0xc36f15c881dd233,
        },
        r0a1: u288 {
            limb0: 0x85e07307b7fb34823e45fcdb,
            limb1: 0xdf691c8433970c52c3fb66e5,
            limb2: 0xb1a4ff27e15f3e0,
        },
        r1a0: u288 {
            limb0: 0x2f10eeddab6ae078977fce0a,
            limb1: 0x6966d12a5e32404d0af9543c,
            limb2: 0x215e4b15d6f2d13,
        },
        r1a1: u288 {
            limb0: 0x580b6777ad088c65b7984621,
            limb1: 0x244baf8ffd127a6305233564,
            limb2: 0x2fbe72b83f00dcf5,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x50bdfb3323d16cbab8f71b5a,
            limb1: 0x39fc43be2032e27d997341bb,
            limb2: 0x1977e90076ddadab,
        },
        r0a1: u288 {
            limb0: 0x83c12a8a9e59891c08a16a40,
            limb1: 0x41a848294eac4b9d5834fbb,
            limb2: 0xe7481412cfd1f7f,
        },
        r1a0: u288 {
            limb0: 0x796851e93fbe9bf84e8c49b8,
            limb1: 0x2bf31380aeac23b18975d34,
            limb2: 0x28f6a6bb4ad98c34,
        },
        r1a1: u288 {
            limb0: 0x4dae372dc078c945ce353c64,
            limb1: 0x57acb87a5a5026380f852e61,
            limb2: 0xbd41ccf3a9fb474,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xae816a8f98326b873f6d099f,
            limb1: 0x564e753240cec9b724c77ffd,
            limb2: 0x118e401882ba14e7,
        },
        r0a1: u288 {
            limb0: 0xd9ed61d4049b3f7ad824d011,
            limb1: 0x3fc74b2b5930b79841d3b34d,
            limb2: 0x23510bb5dff17100,
        },
        r1a0: u288 {
            limb0: 0x72ad0422a4d3462a6331c4d4,
            limb1: 0x841e20719427684114239805,
            limb2: 0x1f6609eb38bffc10,
        },
        r1a1: u288 {
            limb0: 0x7a7d1b5f445f4eb398766631,
            limb1: 0x32bd5710ed947416aecea255,
            limb2: 0x1d6b2665f90d1905,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x23c39d30a11285b02a9b4f12,
            limb1: 0x9c1cac7b2d9393679740b741,
            limb2: 0x1363809e6dcd60a1,
        },
        r0a1: u288 {
            limb0: 0x8317920778c0aebcd6b6c348,
            limb1: 0x88191e606f90267ae7d1cf58,
            limb2: 0x105faf06bdc3493c,
        },
        r1a0: u288 {
            limb0: 0xa4ffff863cd3b3513a5b4186,
            limb1: 0xc6565bba9af10b786471d624,
            limb2: 0x8cbc04a2920f07d,
        },
        r1a1: u288 {
            limb0: 0x3007286d6c49ba12b12ea752,
            limb1: 0x2c136d4cb926005f3a387ded,
            limb2: 0x289132e7517b3044,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x81f257c667c170669479e9a8,
            limb1: 0x261c630d24421c1da48984d8,
            limb2: 0x10d12fe49dedf001,
        },
        r0a1: u288 {
            limb0: 0x478276fcc12974130a40577d,
            limb1: 0xdeb4df7e92e4e5488c312bed,
            limb2: 0x13ccf493e8881180,
        },
        r1a0: u288 {
            limb0: 0xc7ce1840ab96b9f2564d4ff4,
            limb1: 0xd8ae3da33ed9afc891522c40,
            limb2: 0x1805c17c2d4d0b42,
        },
        r1a1: u288 {
            limb0: 0xe11f2969a76def0b30e318d1,
            limb1: 0xa3bd0398e119b42f18445b01,
            limb2: 0x29e5cecefe2e414a,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xdd8ff643dc924e2ae514f595,
            limb1: 0x5db7b1b5877a7fb5ac0541d7,
            limb2: 0x105977c673b0cfba,
        },
        r0a1: u288 {
            limb0: 0xe60de4804ea22708d4a5e55c,
            limb1: 0x23a498b7c7be61fa33a0c524,
            limb2: 0x177302de131f632b,
        },
        r1a0: u288 {
            limb0: 0x8bccefe83b8428f2ecf7b0bd,
            limb1: 0xb5a076a03841eff390a582a2,
            limb2: 0x8485313ab65af53,
        },
        r1a1: u288 {
            limb0: 0xea46ae0351caf62501524c02,
            limb1: 0xd29d5e0656b3496dfca59d12,
            limb2: 0x2e5e53ff8f6186f5,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xfb1091c3e7bba62f63ce81a7,
            limb1: 0xd185d21137f30b904bcadac9,
            limb2: 0x1aa4c3c9f415b032,
        },
        r0a1: u288 {
            limb0: 0xbc9a053328d1b81123e6900b,
            limb1: 0xd1b26e90b4bc5e8044018538,
            limb2: 0x26fae34bba8f737,
        },
        r1a0: u288 {
            limb0: 0xeb7da1fc5e1d36171634f001,
            limb1: 0xd5141add1f058d1294f54738,
            limb2: 0x64bd7f7a0537f6b,
        },
        r1a1: u288 {
            limb0: 0xeefe4d42b425d6b5e4a4eb81,
            limb1: 0xfc0576057c8413ba4b48d99,
            limb2: 0x51286410df152cb,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x76f61c9e4b1149a685c77609,
            limb1: 0x31ce043a1d5f57abe8fddd7,
            limb2: 0x1f08e3fe3b45f4bb,
        },
        r0a1: u288 {
            limb0: 0x176dc0d3a1250356af548d15,
            limb1: 0x9b41f7e50a54e439510ce5aa,
            limb2: 0xbf86e7eede05da6,
        },
        r1a0: u288 {
            limb0: 0xd2e9846f35b544ce542d1b05,
            limb1: 0x317620ddd408212fc7aafb75,
            limb2: 0x15c596f4b4febe58,
        },
        r1a1: u288 {
            limb0: 0x72acd143a48450674d29dd38,
            limb1: 0xaa55bbe70b9a9a511c2bd849,
            limb2: 0x1dc3345a70f473f3,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd05ed478df28a2ccde5c7e27,
            limb1: 0x77ee824b78ab9b898ed01b77,
            limb2: 0x2831206a2e464e3c,
        },
        r0a1: u288 {
            limb0: 0x5e274fa445c4ea9613593214,
            limb1: 0x13099f8b9f64735265437ec,
            limb2: 0x5973e1fe421b082,
        },
        r1a0: u288 {
            limb0: 0xf9cc70ff1a016657b08d372f,
            limb1: 0x158df39c13a865569f13fcd9,
            limb2: 0x1cb53a3058cc46e5,
        },
        r1a1: u288 {
            limb0: 0x7e2b2e4203197570576cfdea,
            limb1: 0xc04b1f21b08384698c27fe70,
            limb2: 0xb9364065a4d13cc,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x3a380c2bee904066ee24547a,
            limb1: 0xc3113b4c60df66e55c72c7c1,
            limb2: 0x26c80957657a7951,
        },
        r0a1: u288 {
            limb0: 0x3f8575e87192e3de5870a60a,
            limb1: 0xccdfbc39cb76cb6705ef0a,
            limb2: 0x26bd3ca5330703d,
        },
        r1a0: u288 {
            limb0: 0x6d01a8051fb61c3726ef189d,
            limb1: 0xfe9314c59844fcc9ea4e2ee5,
            limb2: 0x129241b613460704,
        },
        r1a1: u288 {
            limb0: 0x5198eaadc31620356e6ea78d,
            limb1: 0x27a4bd72ec30bcc3713cf7bd,
            limb2: 0x21b589017bb6318c,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xeacadb90b18b9da590b3fe3d,
            limb1: 0x895e5b680314920a2309f62c,
            limb2: 0x21a0b49c6b5319a1,
        },
        r0a1: u288 {
            limb0: 0xb721d40c354080b5dc2797d3,
            limb1: 0xcf4d74e54c75a81d9fd16be3,
            limb2: 0x16726db9ba2891e1,
        },
        r1a0: u288 {
            limb0: 0xabb86e40b91f32ccf02346fe,
            limb1: 0xc2b48b34a8aff6509018de80,
            limb2: 0x18b2de2aed84e541,
        },
        r1a1: u288 {
            limb0: 0xf71efc0d4495f89f0d0a02d1,
            limb1: 0x31bfb8050a105069fccb1f7f,
            limb2: 0x3036c87ef79aa1ec,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xf4c28b11ce64e3c05926dbfc,
            limb1: 0x4d4a0440e09e3859ec4e83b9,
            limb2: 0x776db9de98c4d00,
        },
        r0a1: u288 {
            limb0: 0x1ec0af0dd4de1b7c42963b24,
            limb1: 0x943ecafeaa0cabec62a17d5,
            limb2: 0x263e82f0f8a5148b,
        },
        r1a0: u288 {
            limb0: 0x46ebf42daea31cbb1095f496,
            limb1: 0x68dbc75a1f5fb33ca6246d2d,
            limb2: 0x1ffc5b2a1fb6b6,
        },
        r1a1: u288 {
            limb0: 0xd9edf3f5f50033c6b4f8f49c,
            limb1: 0xb3b3afb719713a616c10b97a,
            limb2: 0x4b0a19cd33c72b,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x99f8ce28364588aac78235d6,
            limb1: 0x565784fa0220aaf981ab8677,
            limb2: 0x2ad444ad1192f9c6,
        },
        r0a1: u288 {
            limb0: 0xb4bdd3f53febb5cb38294433,
            limb1: 0x86d55035249b985dd1daadf9,
            limb2: 0xc6ace849343f7e2,
        },
        r1a0: u288 {
            limb0: 0x7317e9f0448065a22c4d5705,
            limb1: 0x68a8f12b07b38808b57e01f4,
            limb2: 0xef09af90beaa4e2,
        },
        r1a1: u288 {
            limb0: 0xf2e1df31ea55bf173adc05b8,
            limb1: 0xe05ccba6795e1ce8d2f81b3b,
            limb2: 0x274c88322c783daa,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x6cee7aee1f07d50288cf12b4,
            limb1: 0xd60f34b706c1a3c4382de92f,
            limb2: 0x1d806e382a6d9b6d,
        },
        r0a1: u288 {
            limb0: 0xd51de91ffb08f9f585453589,
            limb1: 0xbd48faa7c34e9a6452d6d199,
            limb2: 0x2dd4301eb1ba91e4,
        },
        r1a0: u288 {
            limb0: 0xddfa46c02befdc9886b51d42,
            limb1: 0x7a8d7a4c65e917035a00aced,
            limb2: 0xabf5334873fd8e7,
        },
        r1a1: u288 {
            limb0: 0x7801d695e887441ded3fda0,
            limb1: 0x976a432df7001dd4525829e6,
            limb2: 0x139073b207a02fd6,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xf61d3c7fa7b512ae95ad106f,
            limb1: 0x3f9afa81234532162c71bf0e,
            limb2: 0x29a5b46507ae5f8,
        },
        r0a1: u288 {
            limb0: 0xbf2e0c099523fbd5955c880c,
            limb1: 0xe14747cd0f404f0563b31088,
            limb2: 0x1c433c8c39f471a6,
        },
        r1a0: u288 {
            limb0: 0xa0d33d38d6342cee605ecca5,
            limb1: 0xb1b84a2250c7ab588a93fdc2,
            limb2: 0x15c6ea0197ad73d6,
        },
        r1a1: u288 {
            limb0: 0xccc68123e9caf8b48015d77a,
            limb1: 0xf9d760126aa7f201b13963b2,
            limb2: 0x1be8f967e5e614a4,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x3e01cc271cb5076e9104a5fd,
            limb1: 0xd4879870cbbef44c586b93e2,
            limb2: 0x1d1d3c72b2f194be,
        },
        r0a1: u288 {
            limb0: 0x4c3692651f7cf9457c35c244,
            limb1: 0x23b835fcf619884737740ae7,
            limb2: 0x152445331a2b3547,
        },
        r1a0: u288 {
            limb0: 0x5f7621016da61d62f805dc32,
            limb1: 0x9dd930551b7bc1d92663448a,
            limb2: 0x16d300277504d9ff,
        },
        r1a1: u288 {
            limb0: 0x6eb5e7c1601ead3f61317f34,
            limb1: 0x95e412c4a605657b9895371c,
            limb2: 0x1e3e32c7ba99270c,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x544ee97d28221f3958f8c9b6,
            limb1: 0xbcba3c8bcae1932cb503dd03,
            limb2: 0x2f80b06e8ae1b258,
        },
        r0a1: u288 {
            limb0: 0xab2348a9874572e8c763d5bf,
            limb1: 0x37fdede2bec39c294d6b9c83,
            limb2: 0x1f86797e6cec8fbe,
        },
        r1a0: u288 {
            limb0: 0xe0c64b1830b74d4f7caef866,
            limb1: 0xa5c4a227c2aeeae8c5131d27,
            limb2: 0x1472bcc94996571,
        },
        r1a1: u288 {
            limb0: 0xf4514e75465858c555185dd0,
            limb1: 0x470b788160d52a41b461295d,
            limb2: 0x252b7e0e867e568c,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xf49f42da6f252da8644d8273,
            limb1: 0x9e3f3882cdf48a8d268cc46e,
            limb2: 0x211c1d34861b2b0,
        },
        r0a1: u288 {
            limb0: 0xe148f1931c3816a315d649db,
            limb1: 0xd0693f80527ab07d23048ca6,
            limb2: 0x2b4beac6818af38e,
        },
        r1a0: u288 {
            limb0: 0x47eb788c964535e5e4123b5c,
            limb1: 0xc65ff341395288b98828ed48,
            limb2: 0xcc0c60d5610424c,
        },
        r1a1: u288 {
            limb0: 0xb3cd59e5845ed3ef524b7a3f,
            limb1: 0x5089598e02c235fb0574a16d,
            limb2: 0x13a0426ed1176e9e,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x1f5a90049fb7110c27fdc78b,
            limb1: 0xdb508f573635368ec0cb46e2,
            limb2: 0x1cfd7d43fb2f70ad,
        },
        r0a1: u288 {
            limb0: 0x4011eda9d928e47b4eb861eb,
            limb1: 0x9bceced968eea2daa0e169c,
            limb2: 0xe9b2e8331df74ca,
        },
        r1a0: u288 {
            limb0: 0xf3ea3193d7e9a8654c1866fd,
            limb1: 0xbc14939571441ef6c73064f7,
            limb2: 0x19ebf4c4b2754201,
        },
        r1a1: u288 {
            limb0: 0x84bc6f8780efa9d730915cec,
            limb1: 0x1253ba3705baac747af08cbb,
            limb2: 0x2256a6367452e4cf,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x37cd1e8de2c96b7f5514e14c,
            limb1: 0x3577dce0b73f3a0e7d73bb18,
            limb2: 0x14badf79609dcb19,
        },
        r0a1: u288 {
            limb0: 0x8f5e9e0e1bb620136e431915,
            limb1: 0x2e224a21d5debdd6932059d8,
            limb2: 0x10b71e9cfb4e2c94,
        },
        r1a0: u288 {
            limb0: 0xf1c093b0b9685dd5bd50b851,
            limb1: 0xd44a54962eff1e08ebc77ffb,
            limb2: 0x2151f2cdd86f324c,
        },
        r1a1: u288 {
            limb0: 0x4cc3c2974b544330e0d23bde,
            limb1: 0x707f1d290d6cc1b4369fc118,
            limb2: 0x265b3c7c14d2e7db,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xfe024ec31d708a71d9d2c7f5,
            limb1: 0x4bf9800e84ce65ec262ae4d9,
            limb2: 0x1acf740627d84ab0,
        },
        r0a1: u288 {
            limb0: 0x201846fd593368e6a1131df6,
            limb1: 0x9b2c36be907376bcc96c4960,
            limb2: 0x358d2be011900b,
        },
        r1a0: u288 {
            limb0: 0x7c0baed0fb20cf51be0059e2,
            limb1: 0x34c5781bae2a99479fd62fb1,
            limb2: 0x2c48b79ada51937b,
        },
        r1a1: u288 {
            limb0: 0x403fc242f33db08c34ab5598,
            limb1: 0xb2c4e7f4f360d4fbf9fa9fea,
            limb2: 0x129615ada22c814a,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x4c95cec60d5006aee713937,
            limb1: 0xf6b602ad62ab8f921ef4bd4,
            limb2: 0x11e559a9b4676b3b,
        },
        r0a1: u288 {
            limb0: 0xbb50d2680c00bc576b5c8e7b,
            limb1: 0x1717b9c3f16362dbc937df2a,
            limb2: 0x185f84310dba94f1,
        },
        r1a0: u288 {
            limb0: 0x4be7713df5b2eaf5a84a6a24,
            limb1: 0x8135369384b5dba991318bb,
            limb2: 0x216d6d9fd74d5714,
        },
        r1a1: u288 {
            limb0: 0xb99501b3a47a9b0cca4d90a0,
            limb1: 0xb35da4b37c5eccc0b3017ecf,
            limb2: 0x191a3a9676714d48,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x39f6903306db1cd3f0039461,
            limb1: 0x4200665edbdb3fd30373ac4,
            limb2: 0x5111c82a5a1feca,
        },
        r0a1: u288 {
            limb0: 0xeb87936fd747cf0712c3bac9,
            limb1: 0x422fb7a08569fa286b5a1ad2,
            limb2: 0x2a959a0c995d8df5,
        },
        r1a0: u288 {
            limb0: 0xfb970a213f304a73d44757e7,
            limb1: 0xa2a89a37afa1eadec65b53ea,
            limb2: 0xb439d7e7e845905,
        },
        r1a1: u288 {
            limb0: 0x7fbb6e2d3d1ba575456c7d18,
            limb1: 0xeeed21111dab7da9ca511e2c,
            limb2: 0xd566c1cdb1c050a,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd1e2eb0bd78f0715fcfffa99,
            limb1: 0x5ba4cf3b71f9dd33d0b309db,
            limb2: 0x123ce881806a565,
        },
        r0a1: u288 {
            limb0: 0xe3d3ddcbeeb46745d43654d5,
            limb1: 0xaa6a91ce1326c5c42f786758,
            limb2: 0x13df4b66ba43088,
        },
        r1a0: u288 {
            limb0: 0x17adaf7302d89e384ffd8468,
            limb1: 0xc5b0283f34e29063d34ff948,
            limb2: 0x11b7619b7de0153a,
        },
        r1a1: u288 {
            limb0: 0x4acae04a40d321cd74b8f051,
            limb1: 0x367db4ab3295d5f6af2af2d5,
            limb2: 0x24528bd119c21449,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xb64e5c38d94eaafe96796192,
            limb1: 0x2adbd7e5c5f5a5b941ac64cf,
            limb2: 0x9059c9db51785b,
        },
        r0a1: u288 {
            limb0: 0x2b0432566502ac889d08e23e,
            limb1: 0x74112ed31882a39276dbc8e6,
            limb2: 0x250534953ff5c07c,
        },
        r1a0: u288 {
            limb0: 0x1a1bb7ca85d67bd132ec540f,
            limb1: 0xfbae2fe60bb300f3bc5740ba,
            limb2: 0x1a45e95423364fc3,
        },
        r1a1: u288 {
            limb0: 0xa84774d6d3cd3a248a310380,
            limb1: 0x771c21b5687c493adb37d8c8,
            limb2: 0x58747a09c5e418,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x7f48605301333fd594aa1c8d,
            limb1: 0x3a66d2653637bd2ff047b6aa,
            limb2: 0x1518cdf95a34c149,
        },
        r0a1: u288 {
            limb0: 0xaf486aa06d8a7118cc1e8f9a,
            limb1: 0xf36d398455b6718870bc8bfb,
            limb2: 0x1075f5438ff2c976,
        },
        r1a0: u288 {
            limb0: 0xf62bd4a7541cbb428f81ff26,
            limb1: 0xec744287b645ddfa0de516c0,
            limb2: 0x1085ae76c984d065,
        },
        r1a1: u288 {
            limb0: 0xd99508da3e4d2f8f30fbb2c0,
            limb1: 0x1e7afa5a519fe698fcd79f88,
            limb2: 0x1d555731fd1af92b,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xfb034cbd0ea061f7f7f2705b,
            limb1: 0x42663e2568918e568b4c9bcc,
            limb2: 0x1963ce5d4c5f911,
        },
        r0a1: u288 {
            limb0: 0xfbcc92509cb92503efc182a9,
            limb1: 0x99af62a685987c8bafad36ec,
            limb2: 0x2f8e9ddc2bcc2cb,
        },
        r1a0: u288 {
            limb0: 0xbae6833b55c62b5da2edeb13,
            limb1: 0x65262a96f5dbbf63980ac9e1,
            limb2: 0x62fc72f0e66d7e7,
        },
        r1a1: u288 {
            limb0: 0xe6a07450acff6c663e94fe18,
            limb1: 0x911346bcfb55760eafab6032,
            limb2: 0x16af741783e5af50,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x69b35b514f325514174433b,
            limb1: 0x250ba204cbd588a6c6ff8f5c,
            limb2: 0x141ac20ccfb82294,
        },
        r0a1: u288 {
            limb0: 0xeb0883dce60bcaf5cd377974,
            limb1: 0x9401138cfa0da42544b00f3d,
            limb2: 0x19464c170483deeb,
        },
        r1a0: u288 {
            limb0: 0xfabe61fd27f000fd15f407f0,
            limb1: 0x7171986bae121a6dee089b82,
            limb2: 0x1da1b0f9ac2ddd03,
        },
        r1a1: u288 {
            limb0: 0x6603337fb112a82b098d48e3,
            limb1: 0xa6bcd293b390bb17894e38cd,
            limb2: 0x1213df846a479a16,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x14f2fb84256d0128365faf00,
            limb1: 0xa03644d859c70c7be40093c9,
            limb2: 0x27a1504b48003ae6,
        },
        r0a1: u288 {
            limb0: 0x81e1cb2e54ed2716e47ebd0c,
            limb1: 0x3cc9bfedda5ba9e3c7d4daf7,
            limb2: 0x28af903e49caf485,
        },
        r1a0: u288 {
            limb0: 0x88f536cd32cb46697a299b5d,
            limb1: 0xab6d2eef1fef6675ca04669d,
            limb2: 0xd7a2c815352d6e2,
        },
        r1a1: u288 {
            limb0: 0xc6dba9d878f2d58a16ba0e3b,
            limb1: 0xb8ae60f84593f8b1f4b4ae9c,
            limb2: 0x1967d5cc94ebab10,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x3d06d61646ffd7575838e78b,
            limb1: 0xdf065736e2f8e3f40c785ac7,
            limb2: 0x2d16988cf5101f59,
        },
        r0a1: u288 {
            limb0: 0x2a6b27684be7df8c1b81d470,
            limb1: 0xf56a0e5b06fd97bd8321e14f,
            limb2: 0xe1c66e17fe968c5,
        },
        r1a0: u288 {
            limb0: 0xf7ea56b48de6788554fa01aa,
            limb1: 0x8cc51ad2c1325b311d2d7d1,
            limb2: 0x2fc95dc9bc835c6f,
        },
        r1a1: u288 {
            limb0: 0xc173dd6c0c2ad9485f07a250,
            limb1: 0x62ad97bb5ae768ffb1294f6f,
            limb2: 0xbed6635e2a3f5cd,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xc57a3a2e57ea8a3fac437fe3,
            limb1: 0x8d04d5a08c075644a3ab4230,
            limb2: 0x1b1e059d08995f6,
        },
        r0a1: u288 {
            limb0: 0x1eb3a71df024500137c95115,
            limb1: 0x2091b93a0010739e87aa4a54,
            limb2: 0x2c3f0d02761636dc,
        },
        r1a0: u288 {
            limb0: 0xb2aee5d259bb6589b1ca0717,
            limb1: 0xad19adf34b126d50c4496862,
            limb2: 0x153135c0cd4176dd,
        },
        r1a1: u288 {
            limb0: 0x3a3931f650c8fa1333d82e0e,
            limb1: 0xb258d880d4ea12951940fe55,
            limb2: 0x1a0d41461bd4f580,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x5c161d9e30e9e40bd2534577,
            limb1: 0xb439f7115b29bff783937414,
            limb2: 0x1d36c659d0a91789,
        },
        r0a1: u288 {
            limb0: 0xe85a3c567b33b5aa146d2e62,
            limb1: 0xa2a7965cdcb4cca8f5d1d2e6,
            limb2: 0x1d358cfdb00ea6ce,
        },
        r1a0: u288 {
            limb0: 0x76e412856955f9b995aaf081,
            limb1: 0x74af08610b1769a6f9f8c9e9,
            limb2: 0xe9e7d69e089558b,
        },
        r1a1: u288 {
            limb0: 0x74a8554cee870c6b8b9b9fdd,
            limb1: 0x3974633d00cc6ecd76ab1746,
            limb2: 0x1356391564722562,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x4c2a70d0445a841898f95ffa,
            limb1: 0xcdbe52a4a70e25df68ad630c,
            limb2: 0x104d49466efb90e6,
        },
        r0a1: u288 {
            limb0: 0x2fa24049d3601b7d58635d84,
            limb1: 0xbbbffe2e27ea96bf12a995a7,
            limb2: 0x18cef9d4307828a5,
        },
        r1a0: u288 {
            limb0: 0xb1624bbd7d5fa1227037147b,
            limb1: 0x1a711e6a56e1ce735bbc9421,
            limb2: 0x239d5d6c3534acb2,
        },
        r1a1: u288 {
            limb0: 0x7172b8bf130829df91f0d6af,
            limb1: 0x5bedaae405cb3a1570c6577d,
            limb2: 0x104f8810a211eacb,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xd48e80e7d51574fcbb96f4d4,
            limb1: 0xd2baf6f6d02ef5f7d8f969ab,
            limb2: 0x1378a55e88ea56e0,
        },
        r0a1: u288 {
            limb0: 0xd6ebddf6ec0bff2cfe24dbcd,
            limb1: 0x2d2c4dc90614e807f2ceb553,
            limb2: 0x1e666ac1f2769cfb,
        },
        r1a0: u288 {
            limb0: 0x8cb00fe5d880a218f1f2a261,
            limb1: 0x6ebb19bddcfaa22ef2eed2b9,
            limb2: 0x11bac11a68216b1e,
        },
        r1a1: u288 {
            limb0: 0x8634a60eeef3fae5bfa0fd70,
            limb1: 0xebcfe6f55c62b6e2468b0dad,
            limb2: 0x1fadb626c155732,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xa229c53ef0bf575adb14b4c8,
            limb1: 0xd94b3edbad8d87c4d647070a,
            limb2: 0x9a58fbbecc5719c,
        },
        r0a1: u288 {
            limb0: 0xcb64ef6492de879f6c0e91a1,
            limb1: 0x881142d69b67e3328179d573,
            limb2: 0x1f3c75c1623edfc5,
        },
        r1a0: u288 {
            limb0: 0x85252974b092d7326f3293db,
            limb1: 0xb60cf0807f393ac20eb7587f,
            limb2: 0x10d55e3e5e149ef7,
        },
        r1a1: u288 {
            limb0: 0x84cbeadbc3c9fb3bffe22c3b,
            limb1: 0xe1bf9032d49f0cbd63a446b0,
            limb2: 0x2c5e7e0243209425,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x37e3490ba7397954255e399b,
            limb1: 0x1088147d12cdee1fc36a7c5,
            limb2: 0x12ccc93970ddfd60,
        },
        r0a1: u288 {
            limb0: 0x69f0537795b398456ae0df32,
            limb1: 0x8681f10773cdf147d1efdee7,
            limb2: 0x1b9e565bc174b283,
        },
        r1a0: u288 {
            limb0: 0x49c28088a7ba75d9967fbda1,
            limb1: 0xf5076dfd9f9fa01b69765b9b,
            limb2: 0x1a39b34d397c8e81,
        },
        r1a1: u288 {
            limb0: 0xa7129cd1e9b9e8a216b70b00,
            limb1: 0x1dc0724da91647a83dc5e644,
            limb2: 0x217434714c670df1,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xf5a59d9e149abdc6e3934598,
            limb1: 0x5762ea1a6103d92b9c1f35e5,
            limb2: 0x14b2d427197b5c3c,
        },
        r0a1: u288 {
            limb0: 0xd44cdf77871dbe2f46008817,
            limb1: 0x176d06dac6d779cefb610a37,
            limb2: 0x4ca140f253cf090,
        },
        r1a0: u288 {
            limb0: 0x2639b8c1d899a6100955b30b,
            limb1: 0x2d7384d20f945a0b282a8e58,
            limb2: 0x274db3ec80605cc1,
        },
        r1a1: u288 {
            limb0: 0x725b0229b3144f88a56a8416,
            limb1: 0xebbc31aef713b1c4f9d2feff,
            limb2: 0x2acf77cafa7e5576,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x91fef9ac578160187341e3fe,
            limb1: 0x37c1273131004ccdab055a7d,
            limb2: 0x1d0667b68b7ebbfb,
        },
        r0a1: u288 {
            limb0: 0xb60c81c0912302ffc7f9128b,
            limb1: 0x127b39dcc05e0cbb3fe56123,
            limb2: 0x2d7a927c80f49378,
        },
        r1a0: u288 {
            limb0: 0x63c8b036104a11503d7d7ab6,
            limb1: 0xfc1395b2b425145fe4ac0c99,
            limb2: 0x26f348661e1520ae,
        },
        r1a1: u288 {
            limb0: 0xf47c5230b33d9457f1ea97a9,
            limb1: 0xbb398fb58026db2cd995bbf5,
            limb2: 0x1ec6df44f7759ff7,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x759bb0dec41bf01dc460c778,
            limb1: 0x46d0baa5ad17b66f69ef78ff,
            limb2: 0x3f186433881bce4,
        },
        r0a1: u288 {
            limb0: 0xafdea96b009de93a5f40fbc2,
            limb1: 0x46ffdba98cd39f787bc9b337,
            limb2: 0x450954848cc8b97,
        },
        r1a0: u288 {
            limb0: 0x1f29db60b2f91c9edeaaf405,
            limb1: 0x3fb6f2ffbd5ecf4209a4bcc,
            limb2: 0x16bb15a1f9a76a96,
        },
        r1a1: u288 {
            limb0: 0xa9e98bdc22bff0ce3c7b3442,
            limb1: 0xe49ce1d5f7cd90a98d50daa,
            limb2: 0x28fad6cfdfd3dec5,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x24c86034bc1b87dc91d21e1b,
            limb1: 0x14d4bbdde00912daa6bc496b,
            limb2: 0xd7a5f3383e67d92,
        },
        r0a1: u288 {
            limb0: 0xdc025d08c7511dbc80367316,
            limb1: 0xcd335f0a0bee496bcfe662ab,
            limb2: 0x211acad86bc52df8,
        },
        r1a0: u288 {
            limb0: 0xe5b983f995c9c32e16839a4a,
            limb1: 0x8c33bc9dd301303b8ecab853,
            limb2: 0x1b90fe2784a45d9a,
        },
        r1a1: u288 {
            limb0: 0x7827b4a174b2a7b77d941c3,
            limb1: 0x4b1fb4eb91927c9a6148a317,
            limb2: 0x1ad1e9af7069003d,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0xfed1d928b70f4b46c7cda13d,
            limb1: 0x7c6428c59296ca5a8abae54b,
            limb2: 0x25af7bff44650c1b,
        },
        r0a1: u288 {
            limb0: 0x549b901b6fed803ceded108c,
            limb1: 0x756b1faf0496e03e7684666,
            limb2: 0x894de134f6b810c,
        },
        r1a0: u288 {
            limb0: 0x2ec074a83ba90a76325ef314,
            limb1: 0x7ede392d3f2232065d9a877c,
            limb2: 0x211f1bdbbdd2943d,
        },
        r1a1: u288 {
            limb0: 0x4def2b1124b0f3b78d87adf,
            limb1: 0xf30d5b512ce4eafbedb2b0b,
            limb2: 0xbe074497371c41,
        },
    },
    G2Line {
        r0a0: u288 {
            limb0: 0x48a061c3ee055d6100780951,
            limb1: 0xd8651dc3dfdc63922ee89cc,
            limb2: 0x2293787a003b8e78,
        },
        r0a1: u288 {
            limb0: 0x373729789ca9a17c3d875d70,
            limb1: 0x581a27ad33989805a1b4b8a5,
            limb2: 0x774c340c87cd37e,
        },
        r1a0: u288 {
            limb0: 0xf424362656c0892b61cd3e05,
            limb1: 0xb13dac4341a7bd428a4760fc,
            limb2: 0x37f953fb901e9b1,
        },
        r1a1: u288 {
            limb0: 0x47c8e8173687d0c13022d5c4,
            limb1: 0x1f7ca039878118cba09ad9eb,
            limb2: 0x29c6b0e07697dd16,
        },
    },
];

