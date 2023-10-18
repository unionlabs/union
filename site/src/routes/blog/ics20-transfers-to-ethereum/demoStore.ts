import { writable } from "svelte/store";
import type { Writable } from "svelte/store";

export const sendingUnoToEthereum: Writable<"start" | "sending" | "done"> =
  writable("start");
export const sendingUnoToUnion: Writable<"start" | "sending" | "done"> =
  writable("start");
