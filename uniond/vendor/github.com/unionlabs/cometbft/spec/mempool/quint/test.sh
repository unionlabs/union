#!/bin/sh

quint test Mempool.qnt
quint run --init consensusInit --step consensusStep --invariant consensusInvariant Consensus.qnt
quint run --init ledgerInit --step ledgerStep --invariant ledgerInvariant Ledger.qnt
quint run --max-steps 30 --invariant allInv --out-itf out.itf Mempool.qnt 
