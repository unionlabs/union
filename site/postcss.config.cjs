const path = require("node:path")
const tailwindcss = require("tailwindcss")
const autoprefixer = require("autoprefixer")
const postcssImport = require("postcss-import")
const postcssNesting = require("tailwindcss/nesting")

/** @type {import('postcss-load-config').Config} */
module.exports = {
  plugins: [
    postcssImport,
    postcssNesting,
    tailwindcss(path.resolve(__dirname, "./tailwind.config.ts")),
    autoprefixer
  ]
}
