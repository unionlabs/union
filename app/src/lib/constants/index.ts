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
      RPC: "https://rpc.testnet-9.union.build",
      // REST: "https://api.testnet.bonlulu.uno"
      REST: "https://rest.testnet-9.union.build/"
    }
  }
}

export const CHAINS = ["SEPOLIA", "UNION"] as const
export type Chain = (typeof CHAINS)[number]
