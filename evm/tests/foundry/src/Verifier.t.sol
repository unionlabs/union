// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.18;

import "forge-std/Test.sol";
import "../../../contracts/core/IZKVerifier.sol";
import "../../../contracts/lib/CometblsHelp.sol";
import "../../../contracts/lib/Encoder.sol";
import "../../../contracts/clients/DevnetVerifier.sol";
import "../../../contracts/proto/ibc/core/channel/v1/channel.sol";
import "../../../contracts/proto/tendermint/types/canonical.sol";

contract VerifierTest is Test {
    using CometblsHelp for TendermintTypesHeader.Data;

    IZKVerifier devnetVerifier;

    function setUp() public {
        devnetVerifier = new DevnetVerifier();
    }

    function testDevnetProof1() public {
        bytes memory blockHash = hex"A589C5189442C696D30502D67D66D4BA5CB39B7D64608216A588DA6DB2F9D3DB";
        bytes memory partSetHeaderHash = hex"532C6BCDE12983D0D593B6100D3F912837C4DC6C2C6F03364B5662958360ED93";
        TendermintTypesCanonicalVote.Data memory vote = TendermintTypesCanonicalVote.Data({
            type_: TendermintTypesTypesGlobalEnums.SignedMsgType.SIGNED_MSG_TYPE_PRECOMMIT,
            height: 1,
            round: 0,
            block_id: TendermintTypesCanonicalBlockID.Data({
                hash: blockHash,
                part_set_header: TendermintTypesCanonicalPartSetHeader.Data({
                    total: 1,
                    hash: partSetHeaderHash
                    })
                }),
            chain_id: "union-devnet-1"
            });
        bytes memory signedVote = Encoder.encodeDelim(TendermintTypesCanonicalVote.encode(vote));
        bytes memory trustedValidatorsHash = hex"D9A17D56267FFEFDEF74C37AA87BF095DECD38B6540DAEACD4FDFE40418F6C89";
        bytes memory untrustedValidatorsHash = hex"D9A17D56267FFEFDEF74C37AA87BF095DECD38B6540DAEACD4FDFE40418F6C89";
        bytes memory zkp = hex"22d7a3190e1eddc1f8e1ffd4ede2b8b3a2952d20a94301d7b8c3ed50b33cbba72bb1021ad4a8bcc14c296bb1c26f4931fdc41700862ace6b9378fe0e6fdcf1ea0f6e628500126a9bf10b6ad8224bdb3da1bba70d7427dd8118f86d1e6b7cdef2006ea2958baab963cd75b4d07def912521aa0b9a0109edfd58c561e1133ebc032b6a6c07fabfbe274fa79eb22562df786c8eadd182aa0ff662bd068ee1702f462e2a1a8935783891085cff462aab85e2e83d6ebbd0938fd6ab40f1aefb7f1e1b1162645b194740f7cc2541bf0d319bfb65ff9ef4fdcb9c3f63bb4fc8d766d7542ba97d40c02df5aaffa016142f8daca0b63a1a99e9b91e80e9a46ec235a5625211d7788d5bd281e237595c203aba336e48baef357d7bccc7c415aa57910f111e1473837b130d67a6ff28e12ab3ae9a77bd66f1d7786985e6d194b15869c94ce104c3abd2fd886a9a9e4d83daf6b292b0c31b0176b54a34ae4d90bd4ed34af355";
        require(
                CometblsHelp.verifyZKP(devnetVerifier, trustedValidatorsHash, untrustedValidatorsHash, signedVote, zkp),
                "invalid proof"
        );
    }

    function testDevnetProof2() public {
        bytes memory blockHash = hex"CF8FB45282F3687C4BF305090C950BC28C7A7A5E35C2A9A1F5930D56A77F3C75";
        bytes memory partSetHeaderHash = hex"39C604A64DDBDA8F2E0F31F0DF30315CE4B8E65DB91F74F29A5ED6926C70A03F";
        TendermintTypesCanonicalVote.Data memory vote = TendermintTypesCanonicalVote.Data({
            type_: TendermintTypesTypesGlobalEnums.SignedMsgType.SIGNED_MSG_TYPE_PRECOMMIT,
            height: 1,
            round: 0,
            block_id: TendermintTypesCanonicalBlockID.Data({
                hash: blockHash,
                part_set_header: TendermintTypesCanonicalPartSetHeader.Data({
                    total: 1,
                    hash: partSetHeaderHash
                    })
                }),
            chain_id: "union-devnet-1"
            });
        bytes memory signedVote = Encoder.encodeDelim(TendermintTypesCanonicalVote.encode(vote));
        bytes memory trustedValidatorsHash = hex"811594B875D1BF0C7992459AE166367C094CB7EAD07DF3F849F4D01EBFBF9A07";
        bytes memory untrustedValidatorsHash = hex"811594B875D1BF0C7992459AE166367C094CB7EAD07DF3F849F4D01EBFBF9A07";
        bytes memory zkp = hex"1024a565ecf146aab820b3f92b98af07ed6e9483aaddae75ed170ddbe4ad650e21f87beb59edd575f4bf87e6ecc5f4e65d969108ba648e827acd47bae6f1f4a62506c89e22c35f465a4a0e6b196bb4b279c8ffbfe3e976c70febc2676ad8f2760b0aebc3e026c5426bedf9ef5a1123dd8791f312fa1b495e84111f59f3b795a82989e1335c662e2e641314c437e5d87688a4f065b95310f722eb7fc033d5f4212a33f014593cbc4cef8e01b9f6b65459d6b0e9d7cc5ddf0b24183fc8260634031d97b77ba34a2c35bd7b615fbb545a47e955619fd25f0d084ea3a273c84189e31cc912765a00b0ad04fef92f04fa5dd88494178e6b940264e41bfacf182e1e310320a02aaf8831d5ca039b6d1fe9adb2959c41ae5c0752750147440c5c7624b9034c55a9e1aabd27fd8edb56200ccd3de9f2dbb7f160ed38fe188d5cf9bb384d010bc55236131830f6eff8d3b9e992a70cbe2ebb9bea52f70b598e9b8eaab713";
        require(
                CometblsHelp.verifyZKP(devnetVerifier, trustedValidatorsHash, untrustedValidatorsHash, signedVote, zkp),
                "invalid proof"
        );
    }

    function testDevnetProof3() public {
        bytes memory partSetHeaderHash = hex"39C604A64DDBDA8F2E0F31F0DF30315CE4B8E65DB91F74F29A5ED6926C70A03F";
        TendermintTypesHeader.Data memory header = TendermintTypesHeader.Data({
            version: TendermintVersionConsensus.Data({
                block: 11,
                app: 0
                }),
            chain_id: "union-devnet-1",
            height: 1,
            time: GoogleProtobufTimestamp.Data({
                secs: 1682000030,
                nanos: 835848794
                }),
            last_block_id: TendermintTypesBlockID.Data({
                hash: bytes(""),
                part_set_header: TendermintTypesPartSetHeader.Data({
                    total: 0,
                    hash: bytes("")
                    })
                }),
            last_commit_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            data_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            validators_hash: hex"811594B875D1BF0C7992459AE166367C094CB7EAD07DF3F849F4D01EBFBF9A07",
            next_validators_hash: hex"811594B875D1BF0C7992459AE166367C094CB7EAD07DF3F849F4D01EBFBF9A07",
            consensus_hash: hex"048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
            app_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            last_results_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            evidence_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            proposer_address: hex"02BE8D2FFE4B72308F7FA92D7C9E6DC3509CD4AD"
            });
        uint256 gasPrevious = gasleft();
        bytes memory blockHash = abi.encodePacked(header.merkleRoot());
        console.log(gasPrevious - gasleft());
        TendermintTypesCanonicalVote.Data memory vote = TendermintTypesCanonicalVote.Data({
            type_: TendermintTypesTypesGlobalEnums.SignedMsgType.SIGNED_MSG_TYPE_PRECOMMIT,
            height: 1,
            round: 0,
            block_id: TendermintTypesCanonicalBlockID.Data({
                hash: blockHash,
                part_set_header: TendermintTypesCanonicalPartSetHeader.Data({
                    total: 1,
                    hash: partSetHeaderHash
                    })
                }),
            chain_id: "union-devnet-1"
            });
        bytes memory signedVote = Encoder.encodeDelim(TendermintTypesCanonicalVote.encode(vote));
        bytes memory trustedValidatorsHash = hex"811594B875D1BF0C7992459AE166367C094CB7EAD07DF3F849F4D01EBFBF9A07";
        bytes memory untrustedValidatorsHash = hex"811594B875D1BF0C7992459AE166367C094CB7EAD07DF3F849F4D01EBFBF9A07";
        bytes memory zkp = hex"1024a565ecf146aab820b3f92b98af07ed6e9483aaddae75ed170ddbe4ad650e21f87beb59edd575f4bf87e6ecc5f4e65d969108ba648e827acd47bae6f1f4a62506c89e22c35f465a4a0e6b196bb4b279c8ffbfe3e976c70febc2676ad8f2760b0aebc3e026c5426bedf9ef5a1123dd8791f312fa1b495e84111f59f3b795a82989e1335c662e2e641314c437e5d87688a4f065b95310f722eb7fc033d5f4212a33f014593cbc4cef8e01b9f6b65459d6b0e9d7cc5ddf0b24183fc8260634031d97b77ba34a2c35bd7b615fbb545a47e955619fd25f0d084ea3a273c84189e31cc912765a00b0ad04fef92f04fa5dd88494178e6b940264e41bfacf182e1e310320a02aaf8831d5ca039b6d1fe9adb2959c41ae5c0752750147440c5c7624b9034c55a9e1aabd27fd8edb56200ccd3de9f2dbb7f160ed38fe188d5cf9bb384d010bc55236131830f6eff8d3b9e992a70cbe2ebb9bea52f70b598e9b8eaab713";
        require(
                CometblsHelp.verifyZKP(devnetVerifier, trustedValidatorsHash, untrustedValidatorsHash, signedVote, zkp),
                "invalid proof"
        );
    }
}
