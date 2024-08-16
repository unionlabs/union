import Navigation from "./navigation.svelte"

const routes = {
  transfer: { draft: false, path: "/transfer" },
  swap: { draft: false, path: "/swap" },
  faucet: { draft: false, path: "/faucet" },
  explorer: { draft: false, path: "/explorer" },
  transfers: { draft: true, path: "/transfers" }
} as const

export { Navigation, routes }
