interface EnvironmentVariables {
  readonly NODE_ENV: "development" | "production" | "test"
  readonly ADDRESS_PREFIX: string
  readonly UNION_COIN_TYPE: string
  readonly UNION_NATIVE_DENOM: string
  readonly HUBBLE_URL: string
}

declare module "bun" {
  interface Env extends EnvironmentVariables {}
}

declare namespace NodeJS {
  interface ProcessEnv extends EnvironmentVariables {}
}
