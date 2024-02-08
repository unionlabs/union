/** @type {import('prettier').Config} */
export default {
  semi: false,
  tabWidth: 2,
  useTabs: false,
  printWidth: 100,
  endOfLine: 'auto',
  singleQuote: true,
  proseWrap: 'never',
  jsxSingleQuote: true,
  arrowParens: 'avoid',
  trailingComma: 'none',
  singleAttributePerLine: true,
  plugins: [
    'prettier-plugin-astro',
    'prettier-plugin-svelte',
    'prettier-plugin-solidity',
    'prettier-plugin-tailwindcss'
  ],
  overrides: [
    { files: '*.astro', options: { parser: 'astro' } },
    { files: '*.sol', options: { parser: 'solidity' } },
    { files: '*.svelte', options: { parser: 'svelte' } },
    { files: '*.mdx', options: { useTabs: false, tabWidth: 2 } }
  ]
}
