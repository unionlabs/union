import { Linter } from "eslint"
import tseslint from "typescript-eslint"
import javascript from "./javascript.js"

const config: Linter.Config[] = [
  ...javascript,
  ...tseslint.configs.recommended as Linter.Config[],
  {
    name: "union:typescript:rules",
    rules: {
      "@typescript-eslint/no-empty-object-type": [
        // Allow `{}` types for empty `TaggedEnum`s
        "off",
      ],
      "@typescript-eslint/no-unused-vars": [
        // Allow unused underscore-prefixed variables
        "error",
        {
          args: "all",
          argsIgnorePattern: "^_",
          caughtErrors: "all",
          caughtErrorsIgnorePattern: "^_",
          destructuredArrayIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          ignoreRestSiblings: true,
        },
      ],
    },
  },
]
export default config
