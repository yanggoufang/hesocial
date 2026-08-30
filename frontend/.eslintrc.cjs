module.exports = {
  root: true,
  env: {
    browser: true,
    es2022: true,
  },
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 'latest',
    sourceType: 'module',
  },
  plugins: ['@typescript-eslint', 'react-hooks', 'react-refresh'],
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:react-hooks/recommended',
  ],
  ignorePatterns: ['dist/'],
  rules: {
    'no-case-declarations': 'warn',
    'no-constant-condition': 'warn',
    'no-empty': 'warn',
    'no-mixed-spaces-and-tabs': 'warn',
    'no-useless-escape': 'warn',
    'prefer-const': 'warn',
    '@typescript-eslint/no-explicit-any': 'off',
    '@typescript-eslint/no-non-null-assertion': 'off',
    '@typescript-eslint/no-unused-vars': ['warn', {
      argsIgnorePattern: '^_',
      caughtErrorsIgnorePattern: '^_',
      varsIgnorePattern: '^_',
    }],
    'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
  },
}
