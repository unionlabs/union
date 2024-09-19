/// <reference types="astro/client" />
/// <reference path="../.astro/env.d.ts" />
/// <reference path="../.astro/types.d.ts" />

interface EventTarget {
  value?: string
}

interface EnvironmentVariables {
  readonly NODE_ENV: "development" | "production" | "test"
  readonly PORT: string
  /* https://union.build in production, http://localhost:${PORT} in development */
  readonly SITE_URL: string
  readonly PUBLIC_ENVIRONMENT: string
  readonly PUBLIC_CONTENTFUL_SPACE_ID: string
  readonly PUBLIC_CONTENTFUL_ENVIRONMENT: string
  readonly PUBLIC_CONTENTFUL_ACCESS_TOKEN: string
  readonly PUBLIC_CONTENTFUL_PREVIEW_TOKEN: string
  readonly PUBLIC_CONTENTFUL_DELIVERY_TOKEN: string
  readonly PUBLIC_CONTENTFUL_WEBHOOK_SECRET: string
  readonly PUBLIC_CONTENTFUL_MANAGEMENT_TOKEN: string
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

declare module "*.splinecode" {
  const content: string
  export default content
}
