# ADR 006: Secret Store Replacement

## Changelog

* July 29th, 2019: Initial draft
* September 11th, 2019: Work has started
* November 4th: Cosmos SDK changes merged in
* November 18th: Gaia changes merged in

## Context

Currently, a Cosmos SDK application's CLI directory stores key material and metadata in a plain text database in the user’s home directory.  Key material is encrypted by a passphrase, protected by bcrypt hashing algorithm. Metadata (e.g. addresses, public keys, key storage details) is available in plain text.

This is not desirable for a number of reasons. Perhaps the biggest reason is insufficient security protection of key material and metadata. Leaking the plain text allows an attacker to surveil what keys a given computer controls via a number of techniques, like compromised dependencies without any privilege execution. This could be followed by a more targeted attack on a particular user/computer.

All modern desktop computers OS (Ubuntu, Debian, MacOS, Windows) provide a built-in secret store that is designed to allow applications to store information that is isolated from all other applications and requires passphrase entry to access the data.

We are seeking solution that provides a common abstraction layer to the many different backends and reasonable fallback for minimal platforms that don’t provide a native secret store.

## Decision

We recommend replacing the current Keybase backend based on LevelDB with [Keyring](https://github.com/99designs/keyring) by 99 designs. This application is designed to provide a common abstraction and uniform interface between many secret stores and is used by AWS Vault application by 99-designs application.

This appears to fulfill the requirement of protecting both key material and metadata from rogue software on a user’s machine.

## Status

Accepted

## Consequences

### Positive

Increased safety for users.

### Negative

Users must manually migrate.

Testing against all supported backends is difficult.

Running tests locally on a Mac require numerous repetitive password entries.

### Neutral

{neutral consequences}

## References

* #4754 Switch secret store to the keyring secret store (original PR by @poldsam) [__CLOSED__]
* #5029 Add support for github.com/99designs/keyring-backed keybases [__MERGED__]
* #5097 Add keys migrate command [__MERGED__]
* #5180 Drop on-disk keybase in favor of keyring [_PENDING_REVIEW_]
* cosmos/gaia#164 Drop on-disk keybase in favor of keyring (gaia's changes) [_PENDING_REVIEW_]
