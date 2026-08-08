import { defineConfig } from 'oxfmt'

export default defineConfig({
  printWidth: 120,
  tabWidth: 2,
  semi: false,
  singleQuote: true,
  trailingComma: 'none',
  arrowParens: 'always',
  endOfLine: 'lf',
  ignorePatterns: [
    'node_modules',
    'dist/**',
    'dist-ssr/**',
    'coverage/**',
    'src-tauri/**',
    'types/auto-imports.d.ts',
    'types/components.d.ts',
    'src/types/global.d.ts'
  ]
})
