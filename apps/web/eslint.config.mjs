// SPDX-License-Identifier: GPL-3.0-or-later
import next from 'eslint-config-next/core-web-vitals';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  { ignores: ['.next/**', 'out/**', 'node_modules/**', 'next-env.d.ts'] },
  tseslint.configs.recommended,
  next,
  {
    rules: {
      // AGENTS.md section 3.1. `any` is not permitted in this codebase; there is no
      // approved use of it. If a type is genuinely unknown, it is `unknown` and you
      // narrow it at the boundary.
      '@typescript-eslint/no-explicit-any': 'error',

      // Silencing the compiler without recording why is how a wrong type survives
      // three refactors. `ts-expect-error` is allowed only with a real explanation.
      '@typescript-eslint/ban-ts-comment': [
        'error',
        {
          'ts-ignore': true,
          'ts-nocheck': true,
          'ts-expect-error': 'allow-with-description',
          minimumDescriptionLength: 10,
        },
      ],

      '@typescript-eslint/no-non-null-assertion': 'error',

      // `consistent-type-imports` is deliberately absent: it needs type-aware linting,
      // and `verbatimModuleSyntax` in tsconfig.json already makes an unmarked type-only
      // import a compile error. One enforcement point, at the stronger layer.
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
    },
  },
);
