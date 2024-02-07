interface EnvironmentVariables {
  readonly NODE_ENV: 'development' | 'production' | 'test'
  readonly PORT: string
  readonly COMMIT_SHA: string
  // from package.json#version
  readonly PUBLIC_VERSION: string
}
// Node.js environment variables types
declare module NodeJS {
  interface ProcessEnv extends EnvironmentVariables {}
}
// Vite environment variables types
interface ImportMetaEnv extends EnvironmentVariables {}
interface ImportMeta {
  readonly env: ImportMetaEnv
}
// Cloudflare Pages/Workers
interface Env extends EnvironmentVariables {}
