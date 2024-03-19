/** @type {import('postcss-load-config').Config} */
module.exports = {
  plugins: [
    require("postcss-import"),
    require("tailwindcss/nesting"),
    require("tailwindcss"),
    require("autoprefixer")
  ]
}
