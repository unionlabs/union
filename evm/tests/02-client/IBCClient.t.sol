pragma solidity ^0.8.18;

import {ILightClient} from "contracts/core/02-client/ILightClient.sol";
import {MockClient} from "contracts/clients/MockClient.sol";
import {IBCMsgs} from "contracts/core/25-handler/IBCMsgs.sol";
import {IBCCommitment} from "contracts/core/24-host/IBCCommitment.sol";

import "forge-std/Test.sol";
import {TestPlus, MsgMocks} from "tests/utils/TestUtils.sol";
import {IBCHandler_Testable} from "tests/utils/IBCHandler_Testable.sol";

contract IBCClientTest is TestPlus {
    IBCHandler_Testable handler;

    string constant CLIENT_TYPE = "mock";
    ILightClient client;
    ILightClient client2;

    constructor() {
        handler = new IBCHandler_Testable();
        client = new MockClient(address(handler));
        client2 = new MockClient(address(handler));
        vm.warp(1);
    }

    //
    // registerClient
    //
    function test_registerClient() public {
        handler.registerClient(CLIENT_TYPE, client);
        handler.registerClient("other", client2);

        assertEq(handler.clientRegistry(CLIENT_TYPE), address(client));
        assertEq(handler.clientRegistry("other"), address(client2));
    }

    function test_registerClient_alreadyRegistered() public {
        handler.registerClient(CLIENT_TYPE, client);

        vm.expectRevert("registerClient: client type already exists");
        handler.registerClient(CLIENT_TYPE, client);
    }

    function test_registerClient_self() public {
        vm.expectRevert("registerClient: must not be self");
        handler.registerClient(CLIENT_TYPE, ILightClient(address(handler)));
    }

    //
    // createClient
    //
    function test_createClient(uint64 revisionHeight) public {
        vm.assume(revisionHeight > 0);

        handler.registerClient(CLIENT_TYPE, client);
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(CLIENT_TYPE, revisionHeight);

        string memory id = handler.createClient(m);

        assertEq(handler.clientTypes(id), m.clientType);
        assertEq(handler.clientImpls(id), address(client));
        // TODO: is this a valid assertion for all client types, or an internal detail of MockClient?
        assertEq(handler.commitments(keccak256(IBCCommitment.clientStatePath(id))), keccak256(m.clientStateBytes));
        assertEq(
            handler.commitments(IBCCommitment.consensusStateCommitmentKey(id, 0, revisionHeight)),
            keccak256(m.consensusStateBytes)
        );
    }

    function test_createClient_noHeight() public {
        handler.registerClient(CLIENT_TYPE, client);
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(CLIENT_TYPE, 0);

        vm.expectRevert("createClient: failed to create client");
        handler.createClient(m);
    }

    function test_createClient_invalidType() public {
        handler.registerClient(CLIENT_TYPE, client);
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient("other", 0);

        vm.expectRevert("createClient: unregistered client type");
        handler.createClient(m);
    }

    //
    // updateClient
    //
    function test_updateClient(uint64 revision, uint64 nextRevision) public {
        vm.assume(revision > 0);
        vm.assume(nextRevision > revision);

        handler.registerClient(CLIENT_TYPE, client);
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(CLIENT_TYPE, revision);

        string memory id = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 = MsgMocks.updateClient(id, nextRevision);
        handler.updateClient(m2);

        assertEq(handler.clientTypes(id), m.clientType);
        assertEq(handler.clientImpls(id), address(client));
        // TODO: assert new commitments
    }

    // TODO: MockClient apparently allows updating with a nextRevision < revision. perhaps we should change this?
    // i.e.: make this test fail
    function test_updateClient_nextRevisionLower(uint64 revision, uint64 nextRevision) public {
        vm.assume(nextRevision > 0);
        vm.assume(nextRevision < revision);

        handler.registerClient(CLIENT_TYPE, client);
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(CLIENT_TYPE, revision);

        string memory id = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 = MsgMocks.updateClient(id, nextRevision);
        handler.updateClient(m2);
    }
}
