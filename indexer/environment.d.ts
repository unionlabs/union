interface EnvironmentVariables {
  readonly NODE_ENV: "development" | "production" | "test";
  readonly UCS01_EVM_ADDRESS: `0x${string}`;
  readonly UNO_ERC20_ADDRESS: `0x${string}`;
  readonly PONDER_RPC_URL_1: string;
  readonly DATABASE_URL?: string;
  readonly PONDER_LOG_LEVEL:
    | "silent"
    | "error"
    | "warn"
    | "info"
    | "debug"
    | "trace";
  readonly INDEX_START_BLOCK: number;
}

declare module "bun" {
  interface Env extends EnvironmentVariables {}
}

declare namespace NodeJS {
  interface ProcessEnv extends EnvironmentVariables {}
}
