/// <reference types="astro/astro-jsx" />

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
