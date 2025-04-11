import prettierConfig from 'eslint-config-prettier';
import pluginPerfectionist from 'eslint-plugin-perfectionist';
import pluginPrettier from 'eslint-plugin-prettier';
import reactHooks from 'eslint-plugin-react-hooks';
import pluginUnicorn from 'eslint-plugin-unicorn';
import globals from 'globals';
import { readFileSync } from 'node:fs';
import tseslint from 'typescript-eslint';

const ignoreList = readFileSync('.prettierignore', 'utf8')
  .split('\n')
  .filter((line) => line.trim() && !line.startsWith('#'));

export default tseslint.config(
  { ignores: ignoreList },
  {
    extends: [...tseslint.configs.recommended],
    files: ['**/*.js', '**/*.mjs', '**/*.ts', '**/*.tsx'],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
    plugins: {
      '@typescript-eslint': tseslint.plugin,
      'react-hooks': reactHooks,
      perfectionist: pluginPerfectionist,
      prettier: pluginPrettier,
      unicorn: pluginUnicorn,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      ...pluginUnicorn.configs.recommended.rules,
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          args: 'all',
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
        },
      ],
      'no-unused-vars': 'off',
      'perfectionist/sort-imports': [
        'error',
        {
          order: 'asc',
          type: 'natural',
        },
      ],
      'prettier/prettier': 'error',
      'unicorn/filename-case': ['error', { case: 'kebabCase' }],
      'unicorn/no-null': 'off',
      'unicorn/prevent-abbreviations': 'off',
      'unicorn/no-abusive-eslint-disable': 'off',
      'unicorn/no-array-reduce': 'off',
      'unicorn/no-static-only-class': 'off',
    },
  },
  prettierConfig,
  {
    files: ['tools/cli/**/*.ts'],
    rules: {
      'unicorn/no-process-exit': 'off',
    },
  },
);
