import { writable } from "svelte/store";
import type { Writable } from "svelte/store";
import type { Tendermint37Client } from "@cosmjs/tendermint-rpc";
import type { SigningStargateClient } from "@cosmjs/stargate";
import type { AccountData, Coin } from "@cosmjs/amino";
import type {
  ApolloClient,
  InMemoryCache,
  NormalizedCacheObject,
} from "@apollo/client";
import type { BrowserProvider, JsonRpcSigner, Signature } from "ethers";
import type { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import type { CosmjsOfflineSigner } from "@leapwallet/cosmos-snap-provider";

export const tendermintClient: Writable<Tendermint37Client | null> =
  writable(null);
export const cosmjsSigner: Writable<CosmjsOfflineSigner | null> =
  writable(null);
export const stargateClient: Writable<SigningStargateClient | null> =
  writable(null);
export const unionAccount: Writable<AccountData | null> = writable(null);
export const apolloClient: Writable<ApolloClient<NormalizedCacheObject> | null> =
  writable(null);
export const ethersProvider: Writable<BrowserProvider | null> = writable(null);
export const ethersSigner: Writable<JsonRpcSigner | null> = writable(null);
export const ethereumAddress: Writable<string | null> = writable(null);
export const cosmwasmClient: Writable<SigningCosmWasmClient | null> =
  writable(null);
export const unionUnoBalance: Writable<Coin | null> = writable(null);
export const ethereumEthBalance: Writable<bigint | null> = writable(null);
export const ethereumUnoBalance: Writable<bigint | null> = writable(null);
export const metamaskInstalled: Writable<boolean> = writable(false);
export const connectedToSepolia: Writable<boolean> = writable(false);
export const snapInstalled: Writable<boolean> = writable(false);
export const connectedToUnion: Writable<boolean> = writable(false);
export const sepUNOAdded: Writable<boolean> = writable(false);
