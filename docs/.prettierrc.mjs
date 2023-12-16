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
  singleAttributePerLine: true,
  overrides: [
    {
      files: '*.mdx',
      options: {
        useTabs: false,
        tabWidth: 2
      }
    }
  ]
}
