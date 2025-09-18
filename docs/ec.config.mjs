import { defineEcConfig } from "@astrojs/starlight/expressive-code"
import { pluginCollapsibleSections } from "@expressive-code/plugin-collapsible-sections"
import { pluginLineNumbers } from "@expressive-code/plugin-line-numbers"
import ecTwoSlash from "expressive-code-twoslash"
import * as ts from "typescript"

export default defineEcConfig({
  logger: true,
  frames: {
    extractFileNameFromCode: true,
    showCopyToClipboardButton: true,
    removeCommentsWhenCopyingTerminalFrames: true,
  },
  defaultProps: {
    showLineNumbers: false,
  },
  useStarlightDarkModeSwitch: true,
  themes: ["github-light", "houston"],
  plugins: [
    ecTwoSlash({
      includeJsDoc: true,
      explicitTrigger: true,
      languages: ["ts", "tsx", "js", "jsx"],
      twoslashOptions: {
        compilerOptions: {
          moduleResolution: ts.ModuleResolutionKind.NodeNext,
          module: ts.ModuleKind.NodeNext,
          target: ts.ScriptTarget.ES2022,
          skipDefaultLibCheck: true,
        },
      },
    }),
    pluginLineNumbers(),
    pluginCollapsibleSections(),
  ],
})
