import ecTwoSlash from "expressive-code-twoslash"
import { pluginLineNumbers } from "@expressive-code/plugin-line-numbers"
import { pluginCollapsibleSections } from "@expressive-code/plugin-collapsible-sections"

/**
 * @typedef {ReturnType<typeof import('@expressive-code/plugin-line-numbers')['pluginLineNumbers']>} ECPlugin
 * @typedef {import('@astrojs/starlight/expressive-code').StarlightExpressiveCodeOptions} EC
 * @typedef {EC & { plugins?: any; defaultProps: EC["defaultProps"] & { showLineNumbers?: boolean } }} ModifiedEC
 */

/** @type {ModifiedEC} */
// @ts-expect-error
export default {
  logger: true,
  frames: {
    extractFileNameFromCode: true,
    showCopyToClipboardButton: true,
    removeCommentsWhenCopyingTerminalFrames: true
  },
  defaultProps: {
    showLineNumbers: false
  },
  useStarlightDarkModeSwitch: true,
  themes: ["github-light", "houston"],
  plugins: [
    ecTwoSlash({
      includeJsDoc: true,
      explicitTrigger: true,
      languages: ["ts", "tsx", "js", "jsx"]
    }),
    pluginLineNumbers(),
    pluginCollapsibleSections()
  ]
}
