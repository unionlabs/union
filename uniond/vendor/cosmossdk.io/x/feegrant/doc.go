/*
Package feegrant provides functionality for authorizing the payment of transaction
fees from one account (key) to another account (key).

Effectively, this allows for a user to pay fees using the balance of an account
different from their own. Example use cases would be allowing a key on a device to
pay for fees using a master wallet, or a third party service allowing users to
pay for transactions without ever really holding their own coins. This package
provides ways for specifying fee allowances such that authorizing fee payment to
another account can be done with clear and safe restrictions.

A user would authorize granting fee payment to another user using
MsgGrantAllowance and revoke that delegation using MsgRevokeAllowance.
In both cases, Granter is the one who is authorizing fee payment and Grantee is
the one who is receiving the fee payment authorization. So grantee would correspond
to the one who is signing a transaction and the granter would be the address that
pays the fees.

The fee allowance that a grantee receives is specified by an implementation of
the FeeAllowance interface. Two FeeAllowance implementations are provided in
this package: BasicAllowance and PeriodicAllowance.
*/
package feegrant
