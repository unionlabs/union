pragma solidity ^0.8.23;

import "forge-std/Test.sol";

import "../../contracts/proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "../../contracts/proto/ibc/lightclients/wasm/v1/wasm.sol";
import "../../contracts/proto/ibc/core/commitment/v1/commitment.sol";
import "../../contracts/proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "../../contracts/proto/tendermint/types/types.sol";
import "../../contracts/proto/tendermint/types/validator.sol";
import "../../contracts/proto/tendermint/types/canonical.sol";
import {CometblsHelp, OptimizedConsensusState} from "../../contracts/lib/CometblsHelp.sol";
import {IZKVerifierV2} from "../../contracts/core/IZKVerifierV2.sol";
import {Verifier} from "../../contracts/clients/Verifier.sol";

contract CometblsHelpProxy {
    function verifyZKP(
        IZKVerifierV2 verifier,
        bytes32 trustedValidatorsHash,
        bytes32 untrustedValidatorsHash,
        bytes memory message,
        bytes memory zkp
    ) public returns (bool) {
        return
            CometblsHelp.verifyZKP(
                verifier,
                trustedValidatorsHash,
                untrustedValidatorsHash,
                message,
                zkp
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

    constructor() {
        proxy = new CometblsHelpProxy();
    }

    function test_verifyZKP_ok() public {
        assertTrue(
            proxy.verifyZKP(
                new Verifier(),
                0x1BBACC23BE35969FFCEFC2892440045E83C3C78E81BF2D6473DD745A93835684,
                0x1BBACC23BE35969FFCEFC2892440045E83C3C78E81BF2D6473DD745A93835684,
                hex"650802113E0200000000000022480A2053D3DC9F43757EEA63FC3B28C383074A111146B2DE7F73A198D29A6D6919DA6D12240801122023C8BED9455A38334F6462A3EAC87616CF51226F825A229FA23CA420E26730B9320E756E696F6E2D6465766E65742D31",
                hex"1BFF5A73FF68B0DAF1F9A29DAD3675762BC346BB36DB701FB5DC80D7FE49C6BE08919D9E16A64159D119278B3BB0EF17DD3703CC3BB6E2FE9E54E84C638EE21829F7FB01533159D70B95F032A99E712D7E06BCCE645D7701615DA5EBAEA75088271F5ECA5E9DAC8D4B1A3CEC0A423F856940D02E5F71B5D9A7C980239987C0D224E1CB5AFE824E7201D959E1B21CBCEE6E0B9BD3B33667741BF7FE201A9A8BA414ACA69E03C3C5DED496B65F02469BC7941AD20E832678BDACF37BD0F7E1A4C50DB32B4A5871AC0584912C09A103FDE335D0A20134C6E2AE52419C84BE775A6C2B4C38DE85412682F66ED405023E9CCEC604B3854B800DFB346EAABF676E844A0EFB9D293D0CC163BAD528AD73B91EDDCDA5FCE49D6ACBA8417D9DC514E1B9FA0C55ADB722120C67CF120CF00225E6B7842A5561772AE1095E30E8FCD7A513370FAD55D57F7794DB096BD7FF7E4373410D04194BE1A1818EFFE5D09F9E08CF8D"
            )
        );
    }

    function test_verifyZKP_ko() public {
        assertFalse(
            proxy.verifyZKP(
                new Verifier(),
                0x09539669AC74E3530F2782443258734DABAC8B8E036ECB9BC378361016892155,
                0x09539669AC74E3530F2782443258734DABAC8B8E036ECB9BC378361016892156,
                hex"650802113E0200000000000022480A20B8C88CE47A2AC003E6736975F3AE14517A07E90CBF3293C8E073CF45FB371E79122408011220AC08E7599D7F5682B77E2293928D68A956B2A73E917E5B9F0DBA64B0ED8C2E2F320E756E696F6E2D6465766E65742D31",
                hex"21D80AACFCA03DC2B84881E3EF1A73C25D2D088E48AA35764A6B4485A78354F021C90A4CBAAB731658D13CE5152F147DF1734F0196031DAF918BF06DAEA1A4E9082959B87795E28482B4FE13AD4B777F9A2D4BFBC8C3FF2640A5DB5619A8F2DA04D6037DAEA584F0C93EDC769859BE695493F48813E491540C37587C2C3214490AE2C9DC087D8039CAF2BD181E289D60EA9AC8B4BF3411A9F9888DC9250525DD055143FE81924CF683CF8381167431A8CB0C984C9DB2BA13D6C9B2374FFD7323052586453C7C06E234B861E9E212EB4A8DF470BD9ADCDB759FED40E62004ECB8210E3A53A0D1F570552C5118521943BC2CC4BB1DA8A5877667A2800D4DF62665304E914F6631B3CE27C88F21E1E8FFAC6C0512D62AE00BEEA79F649BD6E139BD254011571644878C8A72D167D82B5F409360209E1B8E146457C1893769383F4F2F9C0E2EF22885F92672277AF244840CA6EB5298D74E73334BD88360D6B33681"
            )
        );
    }

    function test_optimize_iso(
        uint64 timestamp,
        bytes32 appHash,
        bytes32 validatorsHash
    ) public {
        UnionIbcLightclientsCometblsV1ConsensusState.Data
            memory consensusState = UnionIbcLightclientsCometblsV1ConsensusState
                .Data({
                    timestamp: timestamp,
                    root: IbcCoreCommitmentV1MerkleRoot.Data({
                        hash: abi.encodePacked(appHash)
                    }),
                    next_validators_hash: abi.encodePacked(validatorsHash)
                });
        OptimizedConsensusState memory optimizedConsensusState = proxy.optimize(
            consensusState
        );
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
