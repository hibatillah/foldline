import { describe, expect, test } from "bun:test";

import en from "./en.json";
import id from "./id.json";

/** Lists every leaf path, with array indexes, so a missing item is a missing path. */
function leafPaths(value: unknown, prefix = ""): string[] {
	if (Array.isArray(value)) {
		return value.flatMap((item, index) => leafPaths(item, `${prefix}[${index}]`));
	}
	if (value !== null && typeof value === "object") {
		return Object.entries(value).flatMap(([key, child]) =>
			leafPaths(child, prefix ? `${prefix}.${key}` : key),
		);
	}
	return [prefix];
}

function leafStrings(value: unknown): string[] {
	if (Array.isArray(value)) return value.flatMap(leafStrings);
	if (value !== null && typeof value === "object") return Object.values(value).flatMap(leafStrings);
	return typeof value === "string" ? [value] : [];
}

describe("messages", () => {
	test("id has exactly the keys of en", () => {
		expect(leafPaths(id).toSorted()).toEqual(leafPaths(en).toSorted());
	});

	test("no message promises an unmeasured number", () => {
		const allowed = new Set([
			"23.8 MB",
			"0.3 MB",
			"5.75 MB",
			"17.8 MB",
			"23,8 MB",
			"0,3 MB",
			"24 MB",
		]); // 24 MB is 23.8 MB rounded in the FAQ
		for (const text of [...leafStrings(en), ...leafStrings(id)]) {
			expect(text).not.toContain("50 MB");
			for (const match of text.match(/\d+(?:[.,]\d+)? MB/g) ?? []) {
				expect(allowed.has(match)).toBe(true);
			}
		}
	});
});
