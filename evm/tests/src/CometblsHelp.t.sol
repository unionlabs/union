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
        bytes32 trustedValidatorsHash,
        bytes32 untrustedValidatorsHash,
        bytes memory message,
        bytes memory zkp
    ) public returns (bool) {
        return CometblsHelp.verifyZKP(
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

    function test_verifyZKP_ok_2() public {
        uint256 hashedMessage = CometblsHelp.hashToField(
            hex"650802117c0000000000000022480a20ed341d012b198b8c6962209f30ac4a07c06d53ab258865aade613dcd5800aec5122408011220b1c27e9a68de8ddbc981319dea0ad31aa3e41f6759bd7200581eff9d1373ca9f320e756e696f6e2d6465766e65742d31"
        );
        uint256 trustedValidators = 0x2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d;
        uint256 untrustedValidators = 0x2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d;
        bytes
            memory zkp = hex"07942610e1aeb229308405cd7fc0305f31129bb6d7a3f39b0b18ac0adc09e5301d05c7dfb6b1c21aeeae94928b7ddbf59c04454454b785bc430b8d825dc1a52f0444e6bddff7896fce6625c4fef776be5ef1dc9e539db05241b201e83e1ed2d02942b8a7c777ed5806508ab66547cfcff01f0a0aeffa773f32dfb9bf76c07e700c21088d0ed1f4aea52b7962ac5ffa2748d4b021bcafa5bcec2e1748130e64691dea0ac767b1fd72750c517f49da19aaa4e5e70591f9bdc1d177850275e2f1a90712c5ed8902568d20e34b2f3e224c3bcbefa57917efe64104d19767a419524f1eb315c1291e1eeaaf765a3f3c2f0ddd908b49cd2e5e776dc9b063fa62777dfc2a59e984c4b0a21d8afb790d9d06cb7d0cbef6b573eaa48398a8d0b731f3362c2f385771a9bce77c5e6cd66c074d36b6cb71cfe97c65dd75bcad2a9d91f899ee15256a75d3065bee14962a6b10b05b72ba616034803a76c8487fd9285f502c011eae9d47767324ea7d90ee9b4e8d9dbcad3cdc1759d3566e2351bd1176d3cd28";
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
                new Verifier(),
                0x17204F2B98C9E9A6C92C29AC7E19C1BF025530DEE72793868EE9B040CA00417B,
                0x17204F2B98C9E9A6C92C29AC7E19C1BF025530DEE72793868EE9B040CA00417B,
                hex"650802113E0200000000000022480A207A3675198C63E4D7E49CD290929CA9B713B6FCB867EA023DB55BB9CA505946B212240801122009F221212558CB45E97A3A349E215937FF36B69E1EDE1F468A6C64C71F57A2E4320E756E696F6E2D6465766E65742D31",
                hex"195562CC376E9265A7FD89A086855C100173B717B0DEA58AC9F50120E9CBDD7402D59ADAC8A274C5DDB199915B03B5CFB7A91032A71723876F946A7662135D4912EB1FAD1FCA5E88AD1D9097870391D1D477F4CD2A26F27DB3CFC8B511922C482F374A4821BEE34818589A052995CC5994CE787538207F1BA0D595890EB96D751D947274566F6338FC14BB1728C9E42F47F9D47A8A7F46CFA341D3EC71F0A8E80ECDAA9E38B4D6090989B165E536C4332BDF470E860D85001362EC7B369DE0092FD13C85FE2A16247E574B759B7B8EBFE8C7ED19CE7520A693BD09FD604CA54E2FA277AC176ACEC9626313DA7022E8B8DB599E1B02C25DA90AD508AA315DA67C0EAF8A0F41C4CDC897A4941F3BFA7D0E0C2BDD3030D5B0025FB4030A31C886F417B2509E9ECFEA86AA22F75402599E72C21623E9C32A499D7B14B6DBC3A1251E119244B7DC12B54A74FBC3B23E7954435491D89AFA7ABF6F07E1DADE0B28F0DA1978EC72A2C2C0F1FE8DEDA8DD8DDA7E82454618C3DFF1341C9901456F7E656A"
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
                hex"195562CC376E9265A7FD89A086855C100173B717B0DEA58AC9F50120E9CBDD7402D59ADAC8A274C5DDB199915B03B5CFB7A91032A71723876F946A7662135D4912EB1FAD1FCA5E88AD1D9097870391D1D477F4CD2A26F27DB3CFC8B511922C482F374A4821BEE34818589A052995CC5994CE787538207F1BA0D595890EB96D751D947274566F6338FC14BB1728C9E42F47F9D47A8A7F46CFA341D3EC71F0A8E80ECDAA9E38B4D6090989B165E536C4332BDF470E860D85001362EC7B369DE0092FD13C85FE2A16247E574B759B7B8EBFE8C7ED19CE7520A693BD09FD604CA54E2FA277AC176ACEC9626313DA7022E8B8DB599E1B02C25DA90AD508AA315DA67C0EAF8A0F41C4CDC897A4941F3BFA7D0E0C2BDD3030D5B0025FB4030A31C886F417B2509E9ECFEA86AA22F75402599E72C21623E9C32A499D7B14B6DBC3A1251E119244B7DC12B54A74FBC3B23E7954435491D89AFA7ABF6F07E1DADE0B28F0DA1978EC72A2C2C0F1FE8DEDA8DD8DDA7E82454618C3DFF1341C9901456F7E656A"
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
