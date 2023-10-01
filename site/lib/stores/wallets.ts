import { writable } from 'svelte/store';
import type { Writable } from 'svelte/store';
import type { Tendermint37Client } from '@cosmjs/tendermint-rpc';
import type { SigningStargateClient } from '@cosmjs/stargate';

const tendermintClient: Writable<Tendermint37Client | null> = writable(null);
const stargateClient: Writable<SigningStargateClient | null> = writable(null);
