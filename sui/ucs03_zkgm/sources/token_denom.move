// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's sui subdirectory
//                       The Licensed Work is (c) 2024 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//

// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.

// Notice

// Business Source License 1.1

// Terms

// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.

// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.

// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.

// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.

// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.

// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.

// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).

// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

// module zkgm::fungible_token {
//     use sui::coin::{Self};

//     // one time witness
//     public struct FUNGIBLE_TOKEN has drop {}

//     fun init(witness: FUNGIBLE_TOKEN, ctx: &mut TxContext) {
//         let (treasury_cap, metadata) =
//             coin::create_currency<FUNGIBLE_TOKEN>(
//                 witness,
//                 (@decimals.to_u256()) as u8,
//                 b"muno",
//                 b"muno",
//                 b"zkgm token created by voyager",
//                 option::none(),
//                 ctx
//             );

//         transfer::public_share_object(metadata);
//         transfer::public_transfer(treasury_cap, tx_context::sender(ctx))
//     }
// }

/*

D: 0xAAAABBBB
0000010020232a4f7eb4c2abf5061316704373fd4bffbd297729406d9d04012931405f590b020401f804a11ceb0b060000000a01000e020e1e032c27045308055b5607b101d1010882036006e203380a9a04050c9f042b000a010d020602070212021302140001020001020701000003000c01000103030c0100010504020006050700000b000100010c010601000211030400030808090102040e0b01010c040f0e01010c05100c030001050307040a050d02080007080400020b020108000b030108000105010f010805010b01010900010800070900020a020a020a020b01010805070804020b030109000b02010900010b0201080001090001060804010b03010800020900050c436f696e4d657461646174610e46554e4749424c455f544f4b454e064f7074696f6e0b5472656173757279436170095478436f6e746578740355726c076164647265737304636f696e0f6372656174655f63757272656e63790b64756d6d795f6669656c640e66756e6769626c655f746f6b656e04696e6974046e6f6e65066f7074696f6e137075626c69635f73686172655f6f626a6563740f7075626c69635f7472616e736665720673656e64657207746f5f75323536087472616e736665720a74785f636f6e746578740375726c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002052
000000000000000000000000000000000000000000000000000000000aaaabbbb
0a0205046d756e6f0a020b0a4d554e4f20544f4b454e00020109010000000002140b00070011023307010701070238000a0138010c020c030b0238020b030b012e11063803020002000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020101020000010000232a4f7eb4c2abf5061316704373fd4bffbd297729406d9d04012931405f590b01548762d87d8ccd4c0e3eda2065a2e2cb483a6a6e7c19d0823787571d4fcc3e5c310ed014000000002020814b410fef1581e6c9e966fea2354de6b3cebabe924462a559d10deca09245232a4f7eb4c2abf5061316704373fd4bffbd297729406d9d04012931405f590be803000000000000e0972d000000000000

D: 0xCCCCDDDDEEEEFFFF

0000010020232a4f7eb4c2abf5061316704373fd4bffbd297729406d9d04012931405f590b020401f804a11ceb0b060000000a01000e020e1e032c27045308055b5607b101d1010882036006e203380a9a04050c9f042b000a010d020602070212021302140001020001020701000003000c01000103030c0100010504020006050700000b000100010c010601000211030400030808090102040e0b01010c040f0e01010c05100c030001050307040a050d02080007080400020b020108000b030108000105010f010805010b01010900010800070900020a020a020a020b01010805070804020b030109000b02010900010b0201080001090001060804010b03010800020900050c436f696e4d657461646174610e46554e4749424c455f544f4b454e064f7074696f6e0b5472656173757279436170095478436f6e746578740355726c076164647265737304636f696e0f6372656174655f63757272656e63790b64756d6d795f6669656c640e66756e6769626c655f746f6b656e04696e6974046e6f6e65066f7074696f6e137075626c69635f73686172655f6f626a6563740f7075626c69635f7472616e736665720673656e64657207746f5f75323536087472616e736665720a74785f636f6e746578740375726c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002052000000000000000000000000000000000000000000000000000000000aaaabbbb0a0205046d756e6f0a020b0a4d554e4f20544f4b454e00020109010000000002140b00070011023307010701070238000a0138010c020c030b0238020b030b012e11063803020002000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020101020000010000232a4f7eb4c2abf5061316704373fd4bffbd297729406d9d04012931405f590b01548762d87d8ccd4c0e3eda2065a2e2cb483a6a6e7c19d0823787571d4fcc3e5c310ed014000000002020814b410fef1581e6c9e966fea2354de6b3cebabe924462a559d10deca09245232a4f7eb4c2abf5061316704373fd4bffbd297729406d9d04012931405f590be803000000000000e0972d000000000000

*/
