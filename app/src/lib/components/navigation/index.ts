import Navigation from "./navigation.svelte"

const routes = {
  send: { draft: true, path: "/send" },
  faucet: { draft: false, path: "/faucet" },
  transfers: { draft: true, path: "/transfers" }
} as const

export { Navigation, routes }
