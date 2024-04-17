import { pluginLineNumbers } from "@expressive-code/plugin-line-numbers"
import { pluginCollapsibleSections } from "@expressive-code/plugin-collapsible-sections"

/** @type {import('@astrojs/starlight/expressive-code').StarlightExpressiveCodeOptions['plugins']} */
// @ts-expect-error
const plugins = [pluginLineNumbers(), pluginCollapsibleSections()]

/** @type {import('@astrojs/starlight/expressive-code').StarlightExpressiveCodeOptions} */
export default {
  plugins,
  themes: ["starlight-dark", "starlight-light"],
  styleOverrides: {},
  useStarlightUiThemeColors: true,
  useStarlightDarkModeSwitch: true,
  defaultProps: {
    frame: "none"
  },
  frames: {
    extractFileNameFromCode: true,
    showCopyToClipboardButton: true,
    removeCommentsWhenCopyingTerminalFrames: true
  }
}
