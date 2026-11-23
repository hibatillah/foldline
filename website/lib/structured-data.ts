import { routing } from "@/i18n/routing";

import { absoluteUrl } from "./seo";
import { authorName, authorUrl, repoUrl, siteName, siteUrl } from "./site";

type Locale = (typeof routing.locales)[number];

type StructuredDataInput = {
	locale: Locale;
	description: string;
	faq: { q: string; a: string }[];
};

/** The JSON-LD graph of the home page: the site, the app, its author and the FAQ. */
export function structuredData({ locale, description, faq }: StructuredDataInput) {
	const pageUrl = absoluteUrl(locale);

	return {
		"@context": "https://schema.org",
		"@graph": [
			{
				"@type": "WebSite",
				"@id": `${siteUrl}/#website`,
				url: siteUrl,
				name: siteName,
				inLanguage: routing.locales,
				publisher: { "@id": `${siteUrl}/#author` },
			},
			{
				"@type": "Person",
				"@id": `${siteUrl}/#author`,
				name: authorName,
				url: authorUrl,
			},
			{
				"@type": "SoftwareApplication",
				"@id": `${siteUrl}/#app`,
				name: siteName,
				description,
				url: pageUrl,
				image: `${siteUrl}/icon.png`,
				applicationCategory: "UtilitiesApplication",
				operatingSystem: "Windows 10, Windows 11",
				downloadUrl: `${siteUrl}/api/download`,
				license: "https://www.gnu.org/licenses/gpl-3.0.html",
				isAccessibleForFree: true,
				offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
				author: { "@id": `${siteUrl}/#author` },
				sameAs: [repoUrl],
			},
			{
				"@type": "FAQPage",
				"@id": `${pageUrl}#faq`,
				inLanguage: locale,
				mainEntity: faq.map((item) => ({
					"@type": "Question",
					name: item.q,
					acceptedAnswer: { "@type": "Answer", text: item.a },
				})),
			},
		],
	};
}
