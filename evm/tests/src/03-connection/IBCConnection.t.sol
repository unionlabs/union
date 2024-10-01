pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "../core/IBCHandler.sol";
import "../core/LightClient.sol";

contract IBCConnectionTests is Test {
    bytes32 public constant CLIENT_TYPE = keccak256("zkgm");

    TestIBCHandler handler;
    TestLightClient lightClient;
    uint32 clientId;

    function setUp() public {
        handler = new TestIBCHandler();
        lightClient = new TestLightClient();
        handler.registerClient(CLIENT_TYPE, lightClient);
        clientId = handler.createClient(
            IBCMsgs.MsgCreateClient({
                clientType: CLIENT_TYPE,
                clientStateBytes: hex"CADEBABE",
                consensusStateBytes: hex"DEADC0DE",
                relayer: address(this)
            })
        );
    }

    function test_connectionOpenInit_ok(
        IBCMsgs.MsgConnectionOpenInit calldata msg_
    ) public {
        vm.pauseGasMetering();
        vm.expectEmit();
        emit IBCConnectionLib.ConnectionOpenInit(
            0, msg_.clientId, msg_.counterparty.clientId
        );
        vm.resumeGasMetering();
        handler.connectionOpenInit(msg_);
    }

    function test_connectionOpenInit_commitmentSaved(
        IBCMsgs.MsgConnectionOpenInit calldata msg_
    ) public {
        uint32 connectionId = handler.connectionOpenInit(msg_);
        assertEq(
            handler.commitments(
                IBCCommitment.connectionCommitmentKey(connectionId)
            ),
            keccak256(
                abi.encode(
                    IBCConnection({
                        clientId: msg_.clientId,
                        state: IBCConnectionState.Init,
                        counterparty: msg_.counterparty
                    })
                )
            )
        );
    }

    function test_connectionOpenTry_ok(
        IBCMsgs.MsgConnectionOpenTry memory msg_
    ) public {
        vm.pauseGasMetering();
        msg_.clientId = clientId;
        lightClient.pushValidMembership();
        vm.expectEmit();
        emit IBCConnectionLib.ConnectionOpenTry(
            0,
            msg_.clientId,
            msg_.counterparty.clientId,
            msg_.counterparty.connectionId
        );
        vm.resumeGasMetering();
        handler.connectionOpenTry(msg_);
    }

    function test_connectionOpenTry_clientNotFound(
        IBCMsgs.MsgConnectionOpenTry memory msg_
    ) public {
        vm.assume(msg_.clientId != clientId);
        vm.expectRevert(IBCStoreLib.ErrClientNotFound.selector);
        handler.connectionOpenTry(msg_);
    }

    function test_connectionOpenTry_invalidProof(
        IBCMsgs.MsgConnectionOpenTry memory msg_
    ) public {
        msg_.clientId = clientId;
        vm.expectRevert(IBCConnectionLib.ErrInvalidProof.selector);
        handler.connectionOpenTry(msg_);
    }

    function test_connectionOpenTry_commitmentSaved(
        IBCMsgs.MsgConnectionOpenTry memory msg_
    ) public {
        msg_.clientId = clientId;
        lightClient.pushValidMembership();
        uint32 connectionId = handler.connectionOpenTry(msg_);
        assertEq(
            handler.commitments(
                IBCCommitment.connectionCommitmentKey(connectionId)
            ),
            keccak256(
                abi.encode(
                    IBCConnection({
                        clientId: msg_.clientId,
                        state: IBCConnectionState.TryOpen,
                        counterparty: msg_.counterparty
                    })
                )
            )
        );
    }

    function test_connectionOpenInitOpenAck_ok(
        IBCMsgs.MsgConnectionOpenInit memory msg_,
        IBCMsgs.MsgConnectionOpenAck memory msgAck_
    ) public {
        vm.pauseGasMetering();
        msg_.clientId = clientId;
        uint32 connectionId = handler.connectionOpenInit(msg_);
        msgAck_.connectionId = connectionId;
        lightClient.pushValidMembership();
        vm.expectEmit();
        emit IBCConnectionLib.ConnectionOpenAck(
            msgAck_.connectionId,
            msg_.clientId,
            msg_.counterparty.clientId,
            // The connectionId of the counterpary must be updated after the ack.
            msgAck_.counterpartyConnectionId
        );
        vm.resumeGasMetering();
        handler.connectionOpenAck(msgAck_);
    }

    function test_connectionOpenInitOpenAck_invalidProof(
        IBCMsgs.MsgConnectionOpenInit memory msg_,
        IBCMsgs.MsgConnectionOpenAck memory msgAck_
    ) public {
        msg_.clientId = clientId;
        uint32 connectionId = handler.connectionOpenInit(msg_);
        msgAck_.connectionId = connectionId;
        vm.expectRevert(IBCConnectionLib.ErrInvalidProof.selector);
        handler.connectionOpenAck(msgAck_);
    }

    function test_connectionOpenInitOpenAck_commitmentSaved(
        IBCMsgs.MsgConnectionOpenInit memory msgInit_,
        IBCMsgs.MsgConnectionOpenAck memory msgAck_
    ) public {
        msgInit_.clientId = clientId;
        uint32 connectionId = handler.connectionOpenInit(msgInit_);
        msgAck_.connectionId = connectionId;
        lightClient.pushValidMembership();
        handler.connectionOpenAck(msgAck_);
        // The connectionId of the counterpary must be updated after the ack.
        msgInit_.counterparty.connectionId = msgAck_.counterpartyConnectionId;
        assertEq(
            handler.commitments(
                IBCCommitment.connectionCommitmentKey(connectionId)
            ),
            keccak256(
                abi.encode(
                    IBCConnection({
                        clientId: msgInit_.clientId,
                        state: IBCConnectionState.Open,
                        counterparty: msgInit_.counterparty
                    })
                )
            )
        );
    }

    function test_connectionOpenTryConfirm_ok(
        IBCMsgs.MsgConnectionOpenTry memory msgTry_,
        IBCMsgs.MsgConnectionOpenConfirm memory msgConfirm_
    ) public {
        vm.pauseGasMetering();
        msgTry_.clientId = clientId;
        lightClient.pushValidMembership();
        uint32 connectionId = handler.connectionOpenTry(msgTry_);
        msgConfirm_.connectionId = connectionId;
        lightClient.pushValidMembership();
        vm.expectEmit();
        emit IBCConnectionLib.ConnectionOpenConfirm(
            connectionId,
            msgTry_.clientId,
            msgTry_.counterparty.clientId,
            msgTry_.counterparty.connectionId
        );
        vm.resumeGasMetering();
        handler.connectionOpenConfirm(msgConfirm_);
    }

    function test_connectionOpenTryConfirm_invalidProof(
        IBCMsgs.MsgConnectionOpenTry memory msgTry_,
        IBCMsgs.MsgConnectionOpenConfirm memory msgConfirm_
    ) public {
        msgTry_.clientId = clientId;
        lightClient.pushValidMembership();
        uint32 connectionId = handler.connectionOpenTry(msgTry_);
        msgConfirm_.connectionId = connectionId;
        vm.expectRevert(IBCConnectionLib.ErrInvalidProof.selector);
        handler.connectionOpenConfirm(msgConfirm_);
    }

    function test_connectionOpenTryConfirm_commitmentSaved(
        IBCMsgs.MsgConnectionOpenTry memory msgTry_,
        IBCMsgs.MsgConnectionOpenConfirm memory msgConfirm_
    ) public {
        msgTry_.clientId = clientId;
        lightClient.pushValidMembership();
        uint32 connectionId = handler.connectionOpenTry(msgTry_);
        msgConfirm_.connectionId = connectionId;
        lightClient.pushValidMembership();
        handler.connectionOpenConfirm(msgConfirm_);
        assertEq(
            handler.commitments(
                IBCCommitment.connectionCommitmentKey(connectionId)
            ),
            keccak256(
                abi.encode(
                    IBCConnection({
                        clientId: msgTry_.clientId,
                        state: IBCConnectionState.Open,
                        counterparty: msgTry_.counterparty
                    })
                )
            )
        );
    }
}
