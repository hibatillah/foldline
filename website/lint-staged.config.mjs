export default {
	"*.{ts,tsx,mjs}": ["oxlint --fix", "bun fmt --no-script"],
	"*.{json,css,md}": ["bun fmt --no-script"],
};
