import { writable } from 'svelte/store';
import type { Writable } from 'svelte/store';
import type { Tendermint37Client } from '@cosmjs/tendermint-rpc';
import type { SigningStargateClient } from '@cosmjs/stargate';
import type { AccountData, Coin } from '@cosmjs/amino';
import type { ApolloClient, InMemoryCache, NormalizedCacheObject } from '@apollo/client';
import type { BrowserProvider } from 'ethers';

export const tendermintClient: Writable<Tendermint37Client | null> = writable(null);
export const stargateClient: Writable<SigningStargateClient | null> = writable(null);
export const unionAccount: Writable<AccountData | null> = writable(null);
export const unionBalance: Writable<Coin | null> = writable(null);
export const apolloClient: Writable<ApolloClient<NormalizedCacheObject> | null> = writable(null);
export const ethersProvider: Writable<BrowserProvider | null> = writable(null);
export const ethersSigner: Writable<any | null> = writable(null);
export const ethereumAddress: Writable<string | null> = writable(null);
export const ethereumBalance: Writable<bigint | null> = writable(null);
