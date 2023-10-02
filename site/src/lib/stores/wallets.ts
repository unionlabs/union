import { writable } from 'svelte/store';
import type { Writable } from 'svelte/store';
import type { Tendermint37Client } from '@cosmjs/tendermint-rpc';
import type { SigningStargateClient } from '@cosmjs/stargate';

export const tendermintClient: Writable<Tendermint37Client | null> = writable(null);
export const stargateClient: Writable<SigningStargateClient | null> = writable(null);
