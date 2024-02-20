/** @type {import('prettier').Config} */
export default {
  semi: false,
  tabWidth: 2,
  printWidth: 100,
  endOfLine: 'auto',
  singleQuote: true,
  proseWrap: 'never',
  jsxSingleQuote: true,
  arrowParens: 'avoid',
  trailingComma: 'none',
  bracketSameLine: false,
  singleAttributePerLine: true,
  plugins: [
    'prettier-plugin-svelte',
    'prettier-plugin-tailwindcss' // must come last
  ],
  overrides: [
    {
      files: '*.svelte',
      options: {
        parser: 'svelte',
        plugins: ['prettier-plugin-svelte', 'prettier-plugin-tailwindcss']
      }
    }
  ]
}
