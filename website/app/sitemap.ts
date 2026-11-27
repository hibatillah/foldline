import type { MetadataRoute } from "next";

import { PAGE_KEYS } from "@/lib/pages";
import { routing } from "@/i18n/routing";
import { absoluteUrl } from "@/lib/seo";

/** Pages worth indexing; the download page is left out because it is marked noindex. */
const indexedPaths = [
	"",
	"/releases",
	"/privacy",
	"/terms",
	...PAGE_KEYS.map((key) => `/pages/${key}`),
];

export default function sitemap(): MetadataRoute.Sitemap {
	return indexedPaths.flatMap((path) =>
		routing.locales.map((locale) => ({
			url: absoluteUrl(locale, path),
			alternates: {
				languages: {
					...Object.fromEntries(routing.locales.map((entry) => [entry, absoluteUrl(entry, path)])),
					"x-default": absoluteUrl(routing.defaultLocale, path),
				},
			},
		})),
	);
}
