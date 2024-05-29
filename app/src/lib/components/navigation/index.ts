import Navigation from "./navigation.svelte"

const routes = {
  transfer: { draft: false, path: "/transfer" },
  faucet: { draft: false, path: "/faucet" },
  explorer: {draft: false, path: "/explorer" },
  transfers: { draft: true, path: "/transfers" }
} as const

export { Navigation, routes }
