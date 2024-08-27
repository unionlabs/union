import { pluginLineNumbers } from "@expressive-code/plugin-line-numbers"
import { pluginCollapsibleSections } from "@expressive-code/plugin-collapsible-sections"

/** @type {import('@astrojs/starlight/expressive-code').StarlightExpressiveCodeOptions} */
export default {
  frames: {
    extractFileNameFromCode: true,
    showCopyToClipboardButton: true,
    removeCommentsWhenCopyingTerminalFrames: true
  },
  useStarlightUiThemeColors: true,
  useStarlightDarkModeSwitch: true,
  themes: ["vesper", "rose-pine-dawn"],
  plugins: [pluginLineNumbers(), pluginCollapsibleSections()],
  defaultProps: {
    showLineNumbers: false
  },
  styleOverrides: {
    codeBackground: ({ theme }) => theme.colors["editor.background"]
  }
}
