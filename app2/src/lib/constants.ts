export type Environment = "PRODUCTION" | "STAGING" | "DEVELOPMENT"

export const ENV = (): Environment =>
  window.location.hostname === "btc.union.build" || window.location.hostname === "app.union.build"
    ? "PRODUCTION"
    : window.location.hostname === "staging.btc.union.build" ||
        window.location.hostname === "staging.app.union.build"
      ? "STAGING"
      : "DEVELOPMENT"

export const URLS = () => {
  const GRAPHQL_BASE = "graphql.union.build"

  return {
    GRAPHQL: `https://${GRAPHQL_BASE}/v1/graphql`,
    GRAPHQL_REST: `https://${GRAPHQL_BASE}/api/rest`
  }
}
