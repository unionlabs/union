import eslint from "@eslint/js"
import { Linter } from "eslint"

const config: Linter.Config[] = [
  eslint.configs.recommended,
  {
    name: "union:javascript:rules",
    rules: {},
  },
]
export default config
