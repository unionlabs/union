interface EnvironmentVariables {
  readonly NODE_ENV: "development" | "production" | "test"
  readonly PORT: string
  readonly APP_URL: string
  readonly VERSION: string // from package.json#version
  readonly COMMIT_SHA: string
  readonly SENTRY_AUTH_TOKEN: string
  readonly VITE_APP_VERSION: string
  readonly ETHERSCAN_API_KEY: string
  // `vite-plugin-inspect`
  readonly INSPECT: string
  // `rollup-plugin-visualizer`
  readonly VISUALIZE: string
  readonly DEBUG_TABLE: "true" | "false"
  readonly DEBUG_XSTATE: "true" | "false"
}

// Node.js environment variables types
declare namespace NodeJS {
  interface ProcessEnv extends EnvironmentVariables {}
}

// Vite environment variables types
interface ImportMetaEnv extends EnvironmentVariables {}
interface ImportMeta {
  readonly env: ImportMetaEnv
}
