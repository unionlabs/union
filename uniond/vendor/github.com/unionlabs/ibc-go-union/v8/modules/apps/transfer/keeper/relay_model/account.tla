-------------------------- MODULE account ----------------------------

(**
   The accounts interface; please ignore the definition bodies.
*)

EXTENDS identifiers

CONSTANT
  AccountIds

\* a non-account 
NullAccount == "NullAccount"

\* All accounts
Accounts == { NullAccount }

\* Make an escrow account for the given port and channel
MakeEscrowAccount(port, channel) == NullAccount

\* Make an account from the accound id
MakeAccount(accountId) == NullAccount

\* Type constraints for accounts
AccountTypeOK == 
  /\ NullAccount \in Accounts
  /\ \A p \in Identifiers, c \in Identifiers: 
       MakeEscrowAccount(p, c) \in Accounts
  /\ \A a \in Identifiers:
       MakeAccount(a) \in Accounts

=============================================================================
\* Modification History
\* Last modified Thu Nov 19 18:21:10 CET 2020 by c
\* Last modified Thu Nov 05 14:44:18 CET 2020 by andrey
\* Created Thu Nov 05 13:22:40 CET 2020 by andrey
