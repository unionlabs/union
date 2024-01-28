import { pluginCollapsibleSections } from "@expressive-code/plugin-collapsible-sections";

/** @type {import('@astrojs/starlight/expressive-code').AstroExpressiveCodeOptions} */
export default {
  frames: {
    extractFileNameFromCode: true,
    showCopyToClipboardButton: true,
    removeCommentsWhenCopyingTerminalFrames: true,
  },
  styleOverrides: {},
  plugins: [pluginCollapsibleSections()],
};
