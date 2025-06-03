import * as U from "@unionlabs/eslint-config"
import type { Linter } from "eslint"

const config: Linter.Config[] = [
  ...U.configs.typescript,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        // @ts-ignore
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
]

export default config
