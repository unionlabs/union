-------------------------- MODULE relay ----------------------------
(**
 * A primitive model for account arithmetics and token movement 
 * of the Cosmos SDK ICS20 Token Transfer
 * We completely abstract away many details, 
 * and want to focus on a minimal spec useful for testing
 *
 * We also try to make the model modular in that it uses
 * denomination traces and accounts via abstract interfaces,
 * outlined in denom.tla and account.tla
 *) 

EXTENDS Integers, FiniteSets, Sequences, identifiers, denom_record2, account_record

CONSTANT
  MaxAmount

VARIABLE
  error,
  bank,
  p,  \* we want to start with generating single packets,
  handler,
  history,
  count

Amounts == 0..MaxAmount

GetSourceEscrowAccount(packet) == MakeEscrowAccount(packet.sourcePort, packet.sourceChannel)
GetDestEscrowAccount(packet) == MakeEscrowAccount(packet.destPort, packet.destChannel)

FungibleTokenPacketData == [
  sender: AccountIds,
  receiver: AccountIds,
  denomTrace: DenomTraces,
  amount: Amounts
]

Packets == [
  \* We abstract those packet fields away
  \* sequence: uint64
  \* timeoutHeight: Height
  \* timeoutTimestamp: uint64
  sourcePort: Identifiers,
  sourceChannel: Identifiers,
  destPort: Identifiers,
  destChannel: Identifiers,
  data: FungibleTokenPacketData
]


IsSource(packet) ==
  /\ GetPort(packet.data.denomTrace) = packet.sourcePort
  /\ GetChannel(packet.data.denomTrace) = packet.sourceChannel

\* This function models the port and channel checks that happen when the packet is sent
IsValidSendChannel(packet) ==
  /\ packet.sourcePort = "transfer"
  /\ (packet.sourceChannel = "channel-0" \/ packet.sourceChannel = "channel-1")
  /\ packet.destPort = "transfer"
  /\ packet.destChannel = "channel-0"

\* This function models the port and channel checks that happen when relay gets the packet
IsValidRecvChannel(packet) ==
  /\ packet.sourcePort = "transfer"
  /\ packet.sourceChannel = "channel-0"
  /\ packet.destPort = "transfer"
  /\ (packet.destChannel = "channel-0"  \/ packet.destChannel = "channel-1")


WellFormedPacket(packet) ==
  /\ packet.sourcePort /= NullId
  /\ packet.sourceChannel /= NullId
  /\ packet.destPort /= NullId
  /\ packet.destChannel /= NullId

BankWithAccount(abank, account, denom) ==
    IF <<account, denom>> \in DOMAIN abank
    THEN abank
    ELSE [x \in DOMAIN bank \union { <<account, denom>> }
          |-> IF x = <<account, denom>>
              THEN 0
              ELSE bank[x] ]

IsKnownDenomTrace(trace) ==
  \E account \in Accounts :
     <<account, trace>> \in DOMAIN bank


SendTransferPre(packet, pbank) ==
   LET data == packet.data  
       trace == data.denomTrace  
       sender == data.sender  
       amount == data.amount  
       escrow ==  GetSourceEscrowAccount(packet) 
   IN
   /\ WellFormedPacket(packet)
   /\ IsValidSendChannel(packet)
   /\ IsNativeDenomTrace(trace) \/ (IsValidDenomTrace(trace) /\ IsKnownDenomTrace(trace))
   /\ data.sender /= NullId
   /\ <<escrow, data.denomTrace>> \in DOMAIN pbank  
   /\ \/ amount = 0  \* SendTrasfer actually allows for 0 amount
      \/ <<MakeAccount(sender), trace>> \in DOMAIN pbank /\ bank[MakeAccount(sender), trace] >= amount

SendTransferNext(packet) ==
   LET data == packet.data IN
   LET denom == GetDenom(data.denomTrace) IN
   LET amount == data.amount IN
   LET sender == data.sender IN
   LET escrow == GetSourceEscrowAccount(packet) IN
   LET bankwithescrow == BankWithAccount(bank, escrow, data.denomTrace) IN
   IF SendTransferPre(packet,bankwithescrow)
   THEN
        /\ error' = FALSE
        \*/\ IBCsend(chain, packet)
        /\ IF ~IsSource(packet)
           \* This is how the check is encoded in ICS20 and the implementation.
           \* The meaning is "IF denom = AsAddress(NativeDenom)" because of the following argument:
           \* observe that due to the disjunction in SendTransferPre(packet), we have
           \* ~IsSource(packet) /\ SendTransferPre(packet) => denom = AsAddress(NativeDenom)
           THEN
                \* tokens are from this chain
                \* transfer tokens from sender into escrow account
                bank' = [bankwithescrow EXCEPT ![MakeAccount(sender), data.denomTrace] = @ - amount,
                                     ![escrow, data.denomTrace] = @ + amount]
           ELSE
                \* tokens are from other chain. We forward them.
                \* burn sender's money
                bank' = [bankwithescrow EXCEPT ![MakeAccount(sender), data.denomTrace] = @ - amount]
  ELSE
       /\ error' = TRUE
       /\ UNCHANGED bank


OnRecvPacketPre(packet) ==
  LET data == packet.data
      trace == data.denomTrace
      denom == GetDenom(trace)
      amount == data.amount
  IN
  /\ WellFormedPacket(packet)
  /\ IsValidRecvChannel(packet)
  /\ IsValidDenomTrace(trace)
  /\ amount > 0
     \* if there is no receiver account, it is created by the bank
  /\ data.receiver /= NullId
  /\ IsSource(packet) =>
       LET escrow == GetDestEscrowAccount(packet) IN
       LET denomTrace == ReduceDenomTrace(trace) IN
           /\ <<escrow, denomTrace>> \in DOMAIN bank
           /\ bank[escrow, denomTrace] >= amount


OnRecvPacketNext(packet) ==
   LET data == packet.data IN
   LET trace == data.denomTrace IN
   LET denom == GetDenom(trace) IN
   LET amount == data.amount IN
   LET receiver == data.receiver IN
   /\ IF OnRecvPacketPre(packet)
      THEN
        \* This condition is necessary so that denomination traces do not exceed the maximum length
        /\ (IsSource(packet) \/ TraceLen(trace) < MaxDenomLength)
        /\ error' = FALSE
        /\ IF IsSource(packet)
           THEN
                \* transfer from the escrow account to the receiver account
                LET denomTrace == ReduceDenomTrace(trace) IN
                LET escrow == GetDestEscrowAccount(packet) IN
                LET bankwithreceiver == BankWithAccount(bank, MakeAccount(receiver), denomTrace) IN
                bank' = [bankwithreceiver
                    EXCEPT ![MakeAccount(receiver), denomTrace] = @ + amount,
                           ![escrow, denomTrace] = @ - amount]
           ELSE
                \* create new tokens with new denomination and transfer it to the receiver account
                LET denomTrace == ExtendDenomTrace(packet.destPort, packet.destChannel, trace) IN
                LET bankwithreceiver ==
                    BankWithAccount(bank, MakeAccount(receiver), denomTrace) IN
                bank' = [bankwithreceiver
                    EXCEPT ![MakeAccount(receiver), denomTrace] = @ + amount]
      ELSE
       /\ error' = TRUE
       /\ UNCHANGED bank

       
OnTimeoutPacketPre(packet) ==  
  LET data == packet.data
      trace == data.denomTrace
      denom == GetDenom(trace)
      amount == data.amount
  IN
  /\ WellFormedPacket(packet)
  /\ IsValidSendChannel(packet)
  /\ IsValidDenomTrace(trace)
  /\ data.sender /= NullId
  /\ ~IsSource(packet) =>
       LET escrow == GetSourceEscrowAccount(packet)
       IN  /\ <<escrow, trace>> \in DOMAIN bank
           /\ bank[escrow, trace] >= amount
 

OnTimeoutPacketNext(packet) == 
   LET data == packet.data IN
   LET trace == data.denomTrace IN
   LET denom == GetDenom(data.denomTrace) IN
   LET amount == data.amount IN
   LET sender == data.sender IN
   LET bankwithsender == BankWithAccount(bank, MakeAccount(sender), trace) IN
   IF OnTimeoutPacketPre(packet) 
   THEN  
        /\ error' = FALSE
        /\ IF ~IsSource(packet)
            THEN 
            \* transfer from the escrow acount to the sender account
                \* LET denomsuffix == SubSeq(denom, 3, Len(denom)) IN
               LET escrow == GetSourceEscrowAccount(packet) IN
                bank' = [bankwithsender
                    EXCEPT ![MakeAccount(sender), trace] = @ + amount,
                           ![escrow, trace] = @ - amount]
            ELSE 
            \* mint back the money  
             bank' = [bankwithsender EXCEPT ![MakeAccount(sender), trace] = @ + amount]
   
   ELSE 
       /\ error' = TRUE
       /\ UNCHANGED bank


OnAcknowledgementPacketResultNext(packet) ==
   IF WellFormedPacket(packet)
   THEN
        /\ error' = FALSE
        /\ UNCHANGED bank
   ELSE
        /\ error' = TRUE
        /\ UNCHANGED bank


OnAcknowledgementPacketErrorNext(packet) ==
    OnTimeoutPacketNext(packet)

Init ==
  /\ p \in Packets
  /\ bank = [ x \in {<<NullAccount, NullDenomTrace>>} |-> 0  ]
  /\ count = 0
  /\ history = [
       n \in {0} |-> [
          error |-> FALSE,
          packet |-> p,
          handler |-> "",
          bankBefore |-> bank,
          bankAfter |-> bank
       ]
     ]
  /\ error = FALSE
  /\ handler = ""
  
Next ==
  /\ p' \in Packets
  /\ count'= count + 1
  /\
     \/ (SendTransferNext(p) /\ handler' = "SendTransfer")
     \/ (OnRecvPacketNext(p) /\ handler' = "OnRecvPacket")
     \/ (OnTimeoutPacketNext(p) /\ handler' = "OnTimeoutPacket")
     \/ (OnAcknowledgementPacketResultNext(p) /\ handler' = "OnRecvAcknowledgementResult")
     \/ (OnAcknowledgementPacketErrorNext(p) /\ handler' = "OnRecvAcknowledgementError")
  /\ history' = [ n \in DOMAIN history \union {count'} |->
       IF n = count' THEN
         [ packet |-> p, handler |-> handler', error |-> error', bankBefore |-> bank, bankAfter |-> bank' ]
       ELSE history[n]
     ]

=============================================================================
\* Modification History
\* Last modified Wed Dec 2  10:15:45 CET 2020 by andrey
\* Last modified Fri Nov 20 12:37:38 CET 2020 by c
\* Last modified Thu Nov 05 20:56:37 CET 2020 by andrey
\* Last modified Fri Oct 30 21:52:38 CET 2020 by widder
\* Created Thu Oct 29 20:45:55 CET 2020 by andrey
