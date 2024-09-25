/// <reference types="astro/astro-jsx" />

declare namespace astroHTML.JSX {
  export interface IntrinsicAttributes extends Props {
    children?: Array<Element>
  }
}
