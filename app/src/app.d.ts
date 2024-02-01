// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    // interface PageState {}
    // interface Platform {}
  }
}

interface EnvironmentVariables {
  readonly NODE_ENV: 'development' | 'production' | 'test'
  readonly PORT: string
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

export {}
