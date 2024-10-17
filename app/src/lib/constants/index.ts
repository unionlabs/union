import { sepolia } from "viem/chains"

type Environment = "PRODUCTION" | "STAGING" | "DEVELOPMENT"

export const ENV = (): Environment =>
  window.location.hostname === "app.union.build"
    ? "PRODUCTION"
    : window.location.hostname === "staging.app.union.build"
      ? "STAGING"
      : "DEVELOPMENT"

export const URLS = () => {
  const GRAPHQL_BASE =
    ENV() === "PRODUCTION"
      ? "graphql.union.build"
      : ENV() === "STAGING"
        ? "staging.graphql.union.build"
        : "development.graphql.union.build"

  return {
    GRAPHQL: `https://${GRAPHQL_BASE}/v1/graphql`,
    GRAPHQL_REST: `https://${GRAPHQL_BASE}/api/rest`,
    UNION: {
      /**
       * TODO: add array of RPCs and pass to `viem`'s `fallback` array
       */
      RPC: "https://rpc.testnet-8.union.build",
      // REST: "https://api.testnet.bonlulu.uno"
      REST: "https://rest.testnet-8.union.build/"
    },
    SEPOLIA: {
      RPC: "https://rpc.ankr.com/eth_sepolia/6c72c8d164912bed4694cb882fc4ca55574126511a4f5f66828a53fa2448a20a",
      REST: null
    }
  }
}

export const CHAINS = ["SEPOLIA", "UNION"] as const
export type Chain = (typeof CHAINS)[number]

export const CHAIN = {
  UNION: {
    ID: "union-testnet-8",
    NAME: "Union Testnet"
  },
  SEPOLIA: {
    ID: sepolia.id.toString(),
    NAME: sepolia.name
  }
} satisfies Record<Chain, { ID: string; NAME: string }>

export const CHAIN_URLS = {
  [CHAIN.UNION.ID]: URLS().UNION
}
