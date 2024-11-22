/// <reference types="astro/client" />
/// <reference types="astro/astro-jsx" />
/// <reference path="../.astro/types.d.ts" />
/// <reference types="@astrojs/starlight/types" />
/// <reference types="@astrojs/starlight/schema" />
/// <reference path="@astrojs/starlight/virtual.d.ts" />

interface EventTarget {
  value?: string
  closest(selector: string): Element | null
}

interface EnvironmentVariables {
  readonly NODE_ENV: "development" | "production" | "test"
  readonly PORT: string
  /* https://docs.union.build in production, http://localhost:${PORT} in development */
  readonly SITE_URL: string
  readonly PUBLIC_GRAPHQL_URL: string
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

interface IntrinsicAttributes extends Props {
  children?: Array<Element>
  "client:only"?: "react" | "vue" | "svelte"
}

declare namespace astroHTML.JSX {
  export interface IntrinsicAttributes extends Props {
    children?: Array<Element>
    "client:only"?: "react" | "vue" | "svelte"
  }
}

declare module "icons:astro/*" {
  const component: (props: astroHTML.JSX.SVGAttributes) => astroHTML.JSX.Element
  export default component
}

declare module "icons:svelte/*" {
  import { SvelteComponent } from "svelte"
  import type { SvelteHTMLElements } from "svelte/elements"
  export default class extends SvelteComponent<SvelteHTMLElements["svg"]> {}
}
