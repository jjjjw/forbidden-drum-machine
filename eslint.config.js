import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tseslint from '@typescript-eslint/eslint-plugin'
import parser from '@typescript-eslint/parser'
import react from 'eslint-plugin-react'
import prettier from 'eslint-config-prettier'

export default [
  {
    ignores: ['dist/**', 'src-tauri/**', 'vite.config.ts'],
  },
  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 2020,
      globals: {
        ...globals.browser,
        ...globals.node,
        JSX: 'readonly',
      },
      parser: parser,
      parserOptions: {
        ecmaVersion: 'latest',
        ecmaFeatures: { jsx: true },
        sourceType: 'module',
      },
    },
    settings: { react: { version: '18.3' } },
    plugins: {
      react,
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
      '@typescript-eslint': tseslint,
    },
    rules: {
      ...js.configs.recommended.rules,
      ...react.configs.recommended.rules,
      ...react.configs['jsx-runtime'].rules,
      ...reactHooks.configs.recommended.rules,
      ...tseslint.configs.recommended.rules,
      'react-refresh/only-export-components': [
        'warn',
        { allowConstantExport: true },
      ],
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
      '@typescript-eslint/no-explicit-any': 'warn',
      'react/prop-types': 'off', // TypeScript handles this
      
      // Custom rule to prevent hardcoded command/system/node/event names
      'no-restricted-syntax': [
        'error',
        {
          selector: 'CallExpression[callee.name="invoke"][arguments.0.type="Literal"]',
          message: 'Use Commands constants (e.g., Commands.SendClientEvent) instead of hardcoded command names in invoke calls',
        },
        {
          selector: 'CallExpression[callee.name="invoke"] Property[key.name="systemName"][value.type="Literal"]',
          message: 'Use SystemNames constants instead of hardcoded system names in invoke calls',
        },
        {
          selector: 'CallExpression[callee.name="invoke"] Property[key.name="nodeName"][value.type="Literal"]',
          message: 'Use NodeNames constants instead of hardcoded node names in invoke calls',
        },
        {
          selector: 'CallExpression[callee.name="invoke"] Property[key.name="eventName"][value.type="Literal"]',
          message: 'Use typed event constants (e.g., Auditioner.Kick.Trigger) instead of hardcoded event names in invoke calls',
        },
      ]
    },
  },
  prettier, // Must be last to override conflicting rules
]