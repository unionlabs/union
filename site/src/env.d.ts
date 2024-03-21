/// <reference types="astro/client" />
/// <reference path="../.astro/types.d.ts" />

declare module "*.splinecode" {
  const content: string
  export default content
}

interface EventTarget {
  value?: string
}

interface EnvironmentVariables {
  readonly NODE_ENV: "development" | "production" | "test"
  readonly PORT: string
  /* https://union.build in production, http://localhost:${PORT} in development */
  readonly SITE_URL: string
  /* These are injected by default by Cloudflare Pages */
  readonly CF_PAGES_URL: string
  readonly CF_PAGES_COMMIT_SHA: string
}

// Node.js environment variables types
declare namespace NodeJS {
  interface ProcessEnv extends EnvironmentVariables {}
}

// Vite & Astro environment variables types
interface ImportMetaEnv extends EnvironmentVariables {}
interface ImportMeta {
  readonly env: ImportMetaEnv
}

// Cloudflare Pages/Workers
interface Env extends EnvironmentVariables {}
