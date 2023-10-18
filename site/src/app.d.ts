import { ethers } from "ethers";
import { BrowserProvider, Eip1193Provider } from "ethers/types/providers";

// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    // interface Platform {}
  }
  interface Window {
    ethereum: undefined | (Eip1193Provider & BrowserProvider);
  }
}

export {};
