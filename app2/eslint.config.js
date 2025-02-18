import js from '@eslint/js';
import { includeIgnoreFile } from '@eslint/compat';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import { fileURLToPath } from 'node:url';
import ts from 'typescript-eslint';
const gitignorePath = fileURLToPath(new URL("./.gitignore", import.meta.url));

export default ts.config(
  includeIgnoreFile(gitignorePath),
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs["flat/recommended"],
  {
    languageOptions: {
	  globals: {
	    ...globals.browser,
	    ...globals.node
	  }
	}
  },
  {
    files: ["**/*.svelte"],

    languageOptions: {
	  parserOptions: {
	    parser: ts.parser
	  }
	}
  }
);
