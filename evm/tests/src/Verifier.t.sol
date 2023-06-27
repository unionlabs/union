pragma solidity ^0.8.18;

import "forge-std/Test.sol";
import "../../contracts/core/IZKVerifier.sol";
import "../../contracts/lib/CometblsHelp.sol";
import "../../contracts/lib/Encoder.sol";
import "../../contracts/clients/TestnetVerifier.sol";
import "../../contracts/proto/ibc/core/channel/v1/channel.sol";
import "../../contracts/proto/tendermint/types/canonical.sol";

contract VerifierTest is Test {
    using CometblsHelp for TendermintTypesHeader.Data;
    using CometblsHelp for IZKVerifier;

    IZKVerifier testnetVerifier;

    function setUp() public {
        testnetVerifier = new TestnetVerifier();
    }

    /*
     */
    function testValidTestnetProof() public {
        bytes
            memory blockHash = hex"7022627E60ED78120D2FE8DC7ACDB58A2321B0304F8912D2DFB86CE038E23CA8";
        bytes
            memory partSetHeaderHash = hex"41B8793236EE0980E2EAF1A2FAD268C4A3D8979A0C432F06E284EEC5E74DD69C";
        TendermintTypesCanonicalVote.Data
            memory vote = TendermintTypesCanonicalVote.Data({
                type_: TendermintTypesTypesGlobalEnums
                    .SignedMsgType
                    .SIGNED_MSG_TYPE_PRECOMMIT,
                height: 574,
                round: 0,
                block_id: TendermintTypesCanonicalBlockID.Data({
                    hash: blockHash,
                    part_set_header: TendermintTypesCanonicalPartSetHeader
                        .Data({total: 1, hash: partSetHeaderHash})
                }),
                chain_id: "union-devnet-1"
            });
        bytes memory signedVote = Encoder.encodeDelim(
            TendermintTypesCanonicalVote.encode(vote)
        );
        bytes32 trustedValidatorsHash = hex"1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877";
        bytes32 untrustedValidatorsHash = hex"1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877";
        bytes
            memory zkp = hex"25670583A18A0FA734EE839824AEB2EFAF00F2704178C951B70A01E956C164F32CA7B62707FF3916D88F02F67C1C9334C1EC929F37551212DFCF667903C93C2E0E4D493A02092736D6ADD9A66AAE2B55028FA72FB6137639547BBF4C47EB073E2BB2BE616A4182F3B278C7185E4D21EE535BBA1F44F260D23F869F3E2B3F27400318AAC18834CBDE7001AB47637B05ADDF2C0101CCC1BED2BAB0981AB76225F4212F72E61FED29327F9C81E06DB3C9B67FBF6542BF7742CE807DD0B38134DD652C01BB21CF6B5C01AC3C1E749E9E6859DCD8FAA24C32AC976CD5EF8989E37D6D2896AE7082AC48A94B1BF6BCFCAC412EAD66A22986366C78FA8072060DCC95781159E6255C367EAFBFDAE0C611935C2E6FEEA3F76810FBA9F95FA45700EFA5A017D399707E896688C2CCBB13D014D5189F523D6912AE3D01D0AE5F2EC6B05FA80466F421D4936925454BB6941FD367C93AC498C2CE3503DCB41A58C0C437F39E";
        require(
            testnetVerifier.verifyZKP(
                trustedValidatorsHash,
                untrustedValidatorsHash,
                signedVote,
                zkp
            ),
            "invalid proof"
        );
    }
}
