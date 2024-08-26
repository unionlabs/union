import { pluginLineNumbers } from '@expressive-code/plugin-line-numbers'
import { pluginCollapsibleSections } from "@expressive-code/plugin-collapsible-sections";

/** @type {import('@astrojs/starlight/expressive-code').StarlightExpressiveCodeOptions} */
export default {
  frames: {
    extractFileNameFromCode: true,
    showCopyToClipboardButton: true,
    removeCommentsWhenCopyingTerminalFrames: true
  },
  useStarlightUiThemeColors: true,
  useStarlightDarkModeSwitch: true,
  themes: ["starlight-dark", "starlight-light"],

  plugins: [pluginLineNumbers(), pluginCollapsibleSections()],
  defaultProps: {
    showLineNumbers: false
  }
}
