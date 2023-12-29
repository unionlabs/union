import { BrowserProvider, Eip1193Provider } from "ethers/types/providers";

/**
 * @see https://kit.svelte.dev/docs/types#app
 */
declare global {
  namespace App {}
  interface Window {
    ethereum: undefined | (Eip1193Provider & BrowserProvider);
  }
}

export {};
