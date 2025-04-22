export type Environment = "PRODUCTION" | "STAGING" | "DEVELOPMENT"

export const ENV = (): Environment =>
  window.location.hostname === "btc.union.build" || window.location.hostname === "app.union.build"
    ? "PRODUCTION"
    : window.location.hostname === "staging.btc.union.build" ||
        window.location.hostname === "staging.app.union.build"
      ? "STAGING"
      : window.location.hostname === "localhost" || window.location.hostname === "127.0.0.1"
        ? "DEVELOPMENT"
        : "DEVELOPMENT"

export const URLS = () => {
  const GRAPHQL_BASE = ENV() === "DEVELOPMENT" ? "development.graphql.union.build" : "graphql.union.build"

  return {
    GRAPHQL: `https://${GRAPHQL_BASE}/v1/graphql`,
    GRAPHQL_REST: `https://${GRAPHQL_BASE}/api/rest`
  }
}

export const MAX_MOBILE_SIZE = 768 // in pixels (TODO: use rem)
