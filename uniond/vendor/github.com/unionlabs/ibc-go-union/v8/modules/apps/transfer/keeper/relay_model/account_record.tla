-------------------------- MODULE account_record ----------------------------

(**
   The most basic implementation of accounts, which is a union of normal and escrow accounts
   Represented via records.
*)

EXTENDS identifiers

CONSTANT
  AccountIds

NullAccount == [
  port |-> NullId,
  channel |-> NullId,
  id |-> NullId
]

Accounts == [
  port: Identifiers,
  channel: Identifiers,
  id: AccountIds
]

MakeEscrowAccount(port, channel) == [
  port |-> port,
  channel |-> channel,
  id |-> NullId
]

MakeAccount(accountId) == [
  port |-> NullId,
  channel |-> NullId,
  id |-> accountId
]


ACCOUNT == INSTANCE account
AccountTypeOK == ACCOUNT!AccountTypeOK 


=============================================================================
\* Modification History
\* Last modified Thu Nov 19 18:21:46 CET 2020 by c
\* Last modified Thu Nov 05 14:49:10 CET 2020 by andrey
\* Created Thu Nov 05 13:22:40 CET 2020 by andrey
