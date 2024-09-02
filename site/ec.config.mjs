import { pluginCollapsibleSections } from "@expressive-code/plugin-collapsible-sections";

/** @type {import('@astrojs/starlight/expressive-code').StarlightExpressiveCodeOptions} */
// @ts-expect-error
export default {
  frames: {
    extractFileNameFromCode: true,
    showCopyToClipboardButton: true,
    removeCommentsWhenCopyingTerminalFrames: true,
  },
  styleOverrides: {},
  useStarlightUiThemeColors: true,
  useStarlightDarkModeSwitch: true,
  themes: ["starlight-dark", "starlight-light"],
  plugins: [pluginCollapsibleSections()],
};
