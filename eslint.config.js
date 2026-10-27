import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";

// Mirrors the limits in CLAUDE.md. Keep the two in sync.
const sizeAndComplexityLimits = {
  "max-lines": ["error", { max: 200, skipBlankLines: true, skipComments: true }],
  "max-lines-per-function": ["error", { max: 30, skipBlankLines: true, skipComments: true }],
  "max-params": ["error", 4],
  "max-depth": ["error", 3],
  complexity: ["error", 8],
};

const namingAndClarityRules = {
  "id-length": ["error", { min: 2, exceptions: ["i", "j", "x", "y"] }],
  "no-magic-numbers": ["error", { ignore: [0, 1, -1], ignoreArrayIndexes: true, enforceConst: true }],
  "no-empty": ["error", { allowEmptyCatch: false }],
  "prefer-const": "error",
  eqeqeq: "error",
};

export default tseslint.config(
  { ignores: ["dist", "src-tauri/target", "src/components/ui"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: { globals: globals.browser },
    rules: { ...sizeAndComplexityLimits, ...namingAndClarityRules },
  },
);
