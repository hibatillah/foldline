import { defineConfig } from "oxlint";

// Trimmed from encurehq/website; `bun check` is the source of truth for type errors.
export default defineConfig({
	categories: {
		correctness: "error",
		perf: "warn",
		suspicious: "off",
	},
	options: {
		typeAware: true,
		typeCheck: false,
		maxWarnings: 10,
		reportUnusedDisableDirectives: "warn",
	},
	plugins: ["import", "eslint", "typescript", "unicorn", "oxc", "react", "jsx-a11y", "nextjs"],
	rules: {
		"no-unassigned-import": ["warn", { allow: ["**/*.css"] }],
		"no-console": ["warn", { allow: ["error", "warn"] }],
		"no-alert": "error",
		"no-var": "error",
		eqeqeq: "error",
		"prefer-const": "warn",
		"prefer-template": "error",
		"no-nested-ternary": "error",
		"unicorn/filename-case": ["error", { case: "kebabCase" }],
		"typescript/no-explicit-any": ["error", { ignoreRestArgs: true }],
		"typescript/consistent-type-imports": [
			"warn",
			{ prefer: "type-imports", fixStyle: "inline-type-imports" },
		],
		"react/rules-of-hooks": "error",
		"react/exhaustive-deps": "warn",
		"react/button-has-type": "warn",
		"react/jsx-no-target-blank": "error",
		"jsx-a11y/alt-text": "warn",
		"jsx-a11y/anchor-is-valid": "error",
	},
	ignorePatterns: [
		"**/node_modules/**",
		"**/.next/**",
		"**/bun.lock",
		"next-env.d.ts",
		"components/ui/**",
	],
	env: { es2022: true, node: true, browser: true },
});
