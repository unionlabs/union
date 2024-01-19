/** @type {import('eslint').Linter.Config} */
module.exports = {
  root: true,
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 'latest',
    sourceType: 'module',
    warnOnUnsupportedTypeScriptVersion: true
  },
  env: { node: true, browser: true },
  reportUnusedDisableDirectives: true,
  extends: [
    'eslint:recommended',
    'plugin:unicorn/all',
    'plugin:@typescript-eslint/recommended',
    'plugin:jsonc/prettier',
    'plugin:astro/recommended',
    'prettier'
  ],
  plugins: ['@typescript-eslint', 'prettier'],
  overrides: [
    {
      files: ['*.ts', '*.d.ts'],
      rules: {
        '@typescript-eslint/no-namespace': ['off'],
        '@typescript-eslint/no-explicit-any': ['off']
      }
    },
    {
      files: ['*.mdx', '*.md'],
      extends: 'plugin:mdx/recommended',
      rules: {
        '@typescript-eslint/no-unused-vars': ['off'],
        'no-unused-vars': ['off'],
        'mdx/no-unused-expressions': ['off']
      },
      settings: {
        'mdx/code-blocks': true,
        'mdx/language-mapper': {}
      }
    },
    {
      files: ['*.js', '*.cjs', '*.mjs'],
      /**
       * These rules apply to code blocks in MDX files.
       */
      rules: {
        'no-unused-vars': ['off'],
        '@typescript-eslint/no-unused-vars': ['warn'],
        'no-undef': ['off']
      }
    },
    {
      files: ['*.astro'],
      parser: 'astro-eslint-parser',
      parserOptions: {
        parser: '@typescript-eslint/parser',
        extraFileExtensions: ['.astro']
      },
      rules: {}
    }
  ],
  rules: {
    'unicorn/no-keyword-prefix': ['error', { disallowedPrefixes: ['new', 'for'] }],
    'unicorn/filename-case': ['off'],
    'unicorn/prefer-module': ['off'],
    'unicorn/prefer-spread': ['off'],
    'unicorn/no-array-reduce': ['off'],
    'unicorn/new-for-builtins': ['off'],
    'unicorn/consistent-function-scoping': ['off', { checkArrowFunctions: false }],
    'unicorn/prefer-top-level-await': ['off'],
    'unicorn/prefer-event-target': ['off'],
    'unicorn/prevent-abbreviations': [
      'error',
      {
        allowList: {
          ProcessEnv: true,
          ImportMetaEnv: true,
          Props: true,
          Env: true
        },
        checkFilenames: false
      }
    ],
    'import/no-anonymous-default-export': ['off'],
    'no-unused-vars': ['off'],
    '@typescript-eslint/no-unused-vars': ['off'],
    'array-element-newline': ['error', 'consistent'],
    'object-curly-spacing': ['error', 'always'],
    'prettier/prettier': [
      'warn',
      {},
      {
        usePrettierrc: true,
        fileInfoOptions: {
          withNodeModules: true
        }
      }
    ],
    'no-mixed-operators': ['off'],
    'no-multiple-empty-lines': ['off'],
    'no-unexpected-multiline': ['off'],
    '@typescript-eslint/triple-slash-reference': ['off'],
    '@typescript-eslint/no-var-requires': ['off'],
    '@typescript-eslint/prefer-namespace-keyword': ['off'],
    '@typescript-eslint/no-empty-interface': ['off'],
    '@typescript-eslint/consistent-type-imports': [
      'warn',
      {
        prefer: 'type-imports',
        fixStyle: 'inline-type-imports'
      }
    ],
    '@typescript-eslint/ban-ts-comment': ['off'],
    '@typescript-eslint/ban-types': [
      'warn',
      {
        types: {
          String: {
            message: 'Use string instead',
            fixWith: 'string'
          },

          '{}': false
        }
      }
    ]
  }
}
