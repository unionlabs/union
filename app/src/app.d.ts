// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    // interface PageState {}
    // interface Platform {}
  }
  interface Window {
    EventEmitter: typeof EventEmitter
    ethereum: {
      request(args: { method: EthereumRequestMethod; params?: Record<string, any> }): Promise<any>
    }
  }
}

type EthereumRequestMethod =
  | 'wallet_getSnaps'
  | 'wallet_requestSnaps'
  | 'wallet_invokeSnap'
  | 'wallet_watchAsset'

export {}
