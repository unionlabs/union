-------------------------- MODULE relay_tests ----------------------------

EXTENDS Integers, FiniteSets

Identifiers == {"", "transfer", "channel-0", "channel-1", "cosmos-hub", "ethereum-hub", "bitcoin-hub"}
NullId == ""
MaxAmount == 5
Denoms == {"", "atom", "eth", "btc" }
AccountIds == {"", "a1", "a2", "a3" }

VARIABLES error, bank, p, count, history, handler

INSTANCE relay

\************************** Tests ******************************

\* Generic test for handler pass
TestHandlerPass(handlerName) ==
  \E s \in DOMAIN history :
    /\ history[s].handler = handlerName
    /\ history[s].error = FALSE
    /\ history[s].packet.data.amount > 0

\* Generic test for handler fail
TestHandlerFail(handlerName) ==
  \E s \in DOMAIN history :
    /\ history[s].handler = handlerName
    /\ history[s].error = TRUE
    /\ history[s].packet.data.amount > 0

TestSendTransferPass == TestHandlerPass("SendTransfer")
TestSendTransferPassInv == ~TestSendTransferPass

TestSendTransferFail == TestHandlerFail("SendTransfer")
TestSendTransferFailInv == ~TestSendTransferFail

TestOnRecvPacketPass == TestHandlerPass("OnRecvPacket")
TestOnRecvPacketPassInv == ~TestOnRecvPacketPass

TestOnRecvPacketFail == TestHandlerFail("OnRecvPacket")
TestOnRecvPacketFailInv == ~TestOnRecvPacketFail

TestOnTimeoutPass == TestHandlerPass("OnTimeoutPacket")
TestOnTimeoutPassInv == ~TestOnTimeoutPass

TestOnTimeoutFail == TestHandlerFail("OnTimeoutPacket")
TestOnTimeoutFailInv == ~TestOnTimeoutFail

TestOnRecvAcknowledgementResultPass == TestHandlerPass("OnRecvAcknowledgementResult")
TestOnRecvAcknowledgementResultPassInv == ~TestOnRecvAcknowledgementResultPass

TestOnRecvAcknowledgementResultFail == TestHandlerFail("OnRecvAcknowledgementResult")
TestOnRecvAcknowledgementResultFailInv == ~TestOnRecvAcknowledgementResultFail

TestOnRecvAcknowledgementErrorPass == TestHandlerPass("OnRecvAcknowledgementError")
TestOnRecvAcknowledgementErrorPassInv == ~TestOnRecvAcknowledgementErrorPass

TestOnRecvAcknowledgementErrorFail == TestHandlerFail("OnRecvAcknowledgementError")
TestOnRecvAcknowledgementErrorFailInv == ~TestOnRecvAcknowledgementErrorFail

Test5Packets ==
    count >= 5

Test5PacketsInv == ~Test5Packets

Test5Packets2Different ==
    /\ count >= 5
    /\ \E s1, s2 \in DOMAIN history :
       history[s1].handler /= history[s2].handler

Test5Packets2DifferentInv == ~Test5Packets2Different

Test5PacketsAllDifferent ==
    /\ count >= 5
    /\ \A s1, s2 \in DOMAIN history :
       s1 /= s2 => history[s1].handler /= history[s2].handler

Test5PacketsAllDifferentInv == ~Test5PacketsAllDifferent

Test5PacketsAllDifferentPass ==
    /\ Test5PacketsAllDifferent
    /\ \A s \in DOMAIN history :
       s > 0 =>
         /\ history[s].error = FALSE
         /\ history[s].packet.data.amount > 0

Test5PacketsAllDifferentPassInv == ~Test5PacketsAllDifferentPass

TestUnescrowTokens ==
  \E s \in DOMAIN history :
     /\ IsSource(history[s].packet)
     /\ history[s].handler = "OnRecvPacket"
     /\ history[s].error = FALSE
TestUnescrowTokensInv == ~TestUnescrowTokens

=============================================================================
