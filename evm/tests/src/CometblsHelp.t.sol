pragma solidity ^0.8.23;

import "forge-std/Test.sol";

import "../../contracts/proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "../../contracts/proto/ibc/lightclients/wasm/v1/wasm.sol";
import "../../contracts/proto/ibc/core/commitment/v1/commitment.sol";
import "../../contracts/proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "../../contracts/proto/tendermint/types/types.sol";
import "../../contracts/proto/tendermint/types/validator.sol";
import "../../contracts/proto/tendermint/types/canonical.sol";
import {
    CometblsHelp,
    OptimizedConsensusState
} from "../../contracts/lib/CometblsHelp.sol";
import {IZKVerifierV2} from "../../contracts/core/IZKVerifierV2.sol";
import {Verifier} from "../../contracts/clients/Verifier.sol";

contract CometblsHelpProxy {
    function verifyZKP(
        IZKVerifierV2 verifier,
        bytes memory zkp,
        string memory chainId,
        bytes32 trustedValidatorsHash,
        UnionIbcLightclientsCometblsV1LightHeader.Data memory header
    ) public returns (bool) {
        return
            CometblsHelp.verifyZKP(
                verifier,
                zkp,
                chainId,
                trustedValidatorsHash,
                header
            );
    }

    function optimize(
        UnionIbcLightclientsCometblsV1ConsensusState.Data memory consensusState
    ) public pure returns (OptimizedConsensusState memory) {
        return CometblsHelp.optimize(consensusState);
    }
}

contract CometblsHelpTests is Test {
    CometblsHelpProxy proxy;
    Verifier verifier;

    constructor() {
        proxy = new CometblsHelpProxy();
        verifier = new Verifier();
    }

    function test_verifyZKP_ok_2() public {
        uint256 hashedMessage = CometblsHelp.hashToField(
            hex"650802117c0000000000000022480a20ed341d012b198b8c6962209f30ac4a07c06d53ab258865aade613dcd5800aec5122408011220b1c27e9a68de8ddbc981319dea0ad31aa3e41f6759bd7200581eff9d1373ca9f320e756e696f6e2d6465766e65742d31"
        );
        uint256 trustedValidators =
            0x2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d;
        uint256 untrustedValidators =
            0x2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d;
        bytes memory zkp =
            hex"07942610e1aeb229308405cd7fc0305f31129bb6d7a3f39b0b18ac0adc09e5301d05c7dfb6b1c21aeeae94928b7ddbf59c04454454b785bc430b8d825dc1a52f0444e6bddff7896fce6625c4fef776be5ef1dc9e539db05241b201e83e1ed2d02942b8a7c777ed5806508ab66547cfcff01f0a0aeffa773f32dfb9bf76c07e700c21088d0ed1f4aea52b7962ac5ffa2748d4b021bcafa5bcec2e1748130e64691dea0ac767b1fd72750c517f49da19aaa4e5e70591f9bdc1d177850275e2f1a90712c5ed8902568d20e34b2f3e224c3bcbefa57917efe64104d19767a419524f1eb315c1291e1eeaaf765a3f3c2f0ddd908b49cd2e5e776dc9b063fa62777dfc2a59e984c4b0a21d8afb790d9d06cb7d0cbef6b573eaa48398a8d0b731f3362c2f385771a9bce77c5e6cd66c074d36b6cb71cfe97c65dd75bcad2a9d91f899ee15256a75d3065bee14962a6b10b05b72ba616034803a76c8487fd9285f502c011eae9d47767324ea7d90ee9b4e8d9dbcad3cdc1759d3566e2351bd1176d3cd28";
        assertTrue(
            proxy.verifyZKP(
                new Verifier(),
                bytes32(trustedValidators),
                bytes32(untrustedValidators),
                hex"650802117c0000000000000022480a20ed341d012b198b8c6962209f30ac4a07c06d53ab258865aade613dcd5800aec5122408011220b1c27e9a68de8ddbc981319dea0ad31aa3e41f6759bd7200581eff9d1373ca9f320e756e696f6e2d6465766e65742d31",
                zkp
            )
        );
    }

    function test_verifyZKP_ok() public {
        assertTrue(
            proxy.verifyZKP(
                verifier,
                hex"294A48A750D5C2CF926516752FF484EEBE55FF26CF8A8A7536D98794CF062DB6214D0C9E5C6B164111927A1630889619DBBB40149D8E2D32898E7ACB765542CD0EB8A8E04CCC254C3BFDC2FCE627D59C3C05E2AC76E03977855DD889C1C9BA432FF7FF4DEFCB5286555D36D22DD073A859140508AF9B977F38EB9A604E99A5F6109D43A4AFA0AB161DA2B261DED80FBC0C36E57DE2001338941C834E3262CF751BC1BFC6EC27BB8E106BAAB976285BAC1D4AC38D1B759C8A2852D65CE239974F1275CC6765B3D174FD1122EFDE86137D19F07483FEF5244B1D74B2D9DC598AC32A5CA10E8837FBC89703F4D0D46912CF4AF82341C30C2A1F3941849CC011A56E18AD2162EEB71289B8821CC01875BC1E35E5FC1EBD9114C0B2C0F0D9A96C394001468C70A1716CA98EBE82B1E614D4D9B07292EBAD5B60E0C76FD1D58B485E7D1FB1E07F51A0C68E4CA59A399FCF0634D9585BE478E37480423681B984E96C0A1698D8FCB1DF51CAE023B045E114EED9CB233A5742D9E60E1097206EB20A5058",
                "union-devnet-1337",
                0x1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8,
                UnionIbcLightclientsCometblsV1LightHeader.Data({
                    height: int64(3405691582),
                    time: GoogleProtobufTimestamp.Data({
                        secs: int64(1710783278),
                        nanos: 499600406
                    }),
                    validators_hash: hex"1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8",
                    next_validators_hash: hex"1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8",
                    app_hash: hex"3A34FC963EEFAAE9B7C0D3DFF89180D91F3E31073E654F732340CEEDD77DD25B"
                })
            )
        );
    }

    function test_verifyZKP_tamperedBlock() public {
        assertFalse(
            proxy.verifyZKP(
                verifier,
                hex"294A48A750D5C2CF926516752FF484EEBE55FF26CF8A8A7536D98794CF062DB6214D0C9E5C6B164111927A1630889619DBBB40149D8E2D32898E7ACB765542CD0EB8A8E04CCC254C3BFDC2FCE627D59C3C05E2AC76E03977855DD889C1C9BA432FF7FF4DEFCB5286555D36D22DD073A859140508AF9B977F38EB9A604E99A5F6109D43A4AFA0AB161DA2B261DED80FBC0C36E57DE2001338941C834E3262CF751BC1BFC6EC27BB8E106BAAB976285BAC1D4AC38D1B759C8A2852D65CE239974F1275CC6765B3D174FD1122EFDE86137D19F07483FEF5244B1D74B2D9DC598AC32A5CA10E8837FBC89703F4D0D46912CF4AF82341C30C2A1F3941849CC011A56E18AD2162EEB71289B8821CC01875BC1E35E5FC1EBD9114C0B2C0F0D9A96C394001468C70A1716CA98EBE82B1E614D4D9B07292EBAD5B60E0C76FD1D58B485E7D1FB1E07F51A0C68E4CA59A399FCF0634D9585BE478E37480423681B984E96C0A1698D8FCB1DF51CAE023B045E114EED9CB233A5742D9E60E1097206EB20A5058",
                "union-devnet-1337",
                0x1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8,
                UnionIbcLightclientsCometblsV1LightHeader.Data({
                    height: int64(3405691583),
                    time: GoogleProtobufTimestamp.Data({
                        secs: int64(1710783278),
                        nanos: 499600406
                    }),
                    validators_hash: hex"1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8",
                    next_validators_hash: hex"1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8",
                    app_hash: hex"3A34FC963EEFAAE9B7C0D3DFF89180D91F3E31073E654F732340CEEDD77DD25B"
                })
            )
        );
    }

    function test_optimize_iso(
        uint64 timestamp,
        bytes32 appHash,
        bytes32 validatorsHash
    ) public {
        UnionIbcLightclientsCometblsV1ConsensusState.Data memory consensusState =
        UnionIbcLightclientsCometblsV1ConsensusState.Data({
            timestamp: timestamp,
            root: IbcCoreCommitmentV1MerkleRoot.Data({
                hash: abi.encodePacked(appHash)
            }),
            next_validators_hash: abi.encodePacked(validatorsHash)
        });
        OptimizedConsensusState memory optimizedConsensusState =
            proxy.optimize(consensusState);
        assertEq(consensusState.timestamp, optimizedConsensusState.timestamp);
        assertEq(
            consensusState.root.hash,
            abi.encodePacked(optimizedConsensusState.appHash)
        );
        assertEq(
            consensusState.next_validators_hash,
            abi.encodePacked(optimizedConsensusState.nextValidatorsHash)
        );
    }
}
