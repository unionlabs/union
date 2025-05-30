import type { Config as SvelteConfig } from "@sveltejs/kit"
import type { Linter } from "eslint"
import svelte from "eslint-plugin-svelte"
import globals from "globals"
import tseslint from "typescript-eslint"
import javascript from "./javascript.js"
import typescript from "./typescript.js"

const config = (config: SvelteConfig): Linter.Config[] => [
  ...javascript,
  ...typescript,
  ...svelte.configs["flat/recommended"],
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts", "**/*.svelte.js"],

    languageOptions: {
      parserOptions: {
        projectService: true,
        extraFileExtensions: [".svelte"], // Add support for additional file extensions, such as .svelte
        parser: tseslint.parser,
        // Specify a parser for each language, if needed:
        // parser: {
        //   ts: ts.parser,
        //   js: espree,    // Use espree for .js files (add: import espree from 'espree')
        //   typescript: ts.parser
        // },

        // We recommend importing and specifying svelte.config.js.
        // By doing so, some rules in eslint-plugin-svelte will automatically read the configuration and adjust their behavior accordingly.
        // While certain Svelte settings may be statically loaded from svelte.config.js even if you don’t specify it,
        // explicitly specifying it ensures better compatibility and functionality.
        config,
      },
    },
  },
  {
    name: "union:svelte:rules",
    rules: {},
  },
]
export default config
