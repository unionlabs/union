import { writable } from 'svelte/store';
import type { Writable } from 'svelte/store';

export const sendingUnoToEthereum: Writable<'start' | 'sending' | 'done'> = writable('start');
