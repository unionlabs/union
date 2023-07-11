pragma solidity ^0.8.18;

import {ILightClient} from "contracts/core/02-client/ILightClient.sol";
import {MockClient} from "contracts/clients/MockClient.sol";
import {IBCMsgs} from "contracts/core/25-handler/IBCMsgs.sol";
import {IBCCommitment} from "contracts/core/24-host/IBCCommitment.sol";
import {IBCStore} from "contracts/core/24-host/IBCStore.sol";

import "forge-std/Test.sol";
import {TestPlus, MsgMocks} from "tests/utils/TestUtils.sol";
import {IBCHandler_Testable} from "tests/utils/IBCHandler_Testable.sol";

contract IBCClientHandlerTest is TestPlus {
    // TODO: since IBCClientHandler is just a proxy to IBCClient, it should inherit from IIBCClient interface, but doesn't
    IBCHandler_Testable handler;

    string constant CLIENT_TYPE = "mock";
    ILightClient client;
    ILightClient client2;

    event GeneratedClientIdentifier(string);

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

        // TODO: no event emitted here?
    }

    //
    // createClient
    //
    function test_createClient(uint64 revisionHeight) public {
        vm.assume(revisionHeight > 0);

        handler.registerClient(CLIENT_TYPE, client);
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(CLIENT_TYPE, revisionHeight);

        vm.expectEmit(false, false, false, false);
        emit GeneratedClientIdentifier("");
        string memory id = handler.createClient(m);
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

        // TODO: no event emitted here?
    }
}
