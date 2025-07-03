const path = require("node:path")
const autoprefixer = require("autoprefixer")
const postcssImport = require("postcss-import")

/** @type {import('postcss-load-config').Config} */
module.exports = {
  plugins: [
    postcssImport,
    autoprefixer,
  ],
}
