import type { Metadata } from "next";

import { routing } from "@/i18n/routing";

import { openGraphLocales, siteName, siteUrl } from "./site";

type Locale = (typeof routing.locales)[number];

/** The path of a page in a locale: English has no prefix, every other locale does. */
export function localePath(locale: Locale, path = ""): string {
	if (locale === routing.defaultLocale) return path || "/";
	return `/${locale}${path}`;
}

/** The first sentence of a text, for a meta description that must stay short. */
export function firstSentence(text: string): string {
	const end = text.indexOf(". ");
	return end === -1 ? text : text.slice(0, end + 1);
}

/** The one share image of every page; `app/opengraph-image.jpg` and `twitter-image.jpg` are the same picture. */
const shareImage = {
	url: "/opengraph-image.jpg",
	width: 1200,
	height: 630,
	alt: "Foldline: a Dynamic Island notch for the top of your Windows screen, shown as a grid of its pages and messages",
};

type PageMetadataInput = {
	locale: Locale;
	/** The page path without the locale prefix: "" for the home page, "/privacy" for the policy. */
	path?: string;
	title: string;
	description: string;
	/** The text of the Open Graph and Twitter cards when it differs from the page description. */
	shareDescription?: string;
	/** The home page title already holds the site name, so the layout title template is skipped. */
	absoluteTitle?: boolean;
	index?: boolean;
};

/** Title, description, canonical, language alternates, Open Graph and Twitter tags of one page. */
export function pageMetadata({
	locale,
	path = "",
	title,
	description,
	shareDescription = description,
	absoluteTitle = false,
	index = true,
}: PageMetadataInput): Metadata {
	const url = localePath(locale, path);
	const languages: Record<string, string> = Object.fromEntries(
		routing.locales.map((entry) => [entry, localePath(entry, path)]),
	);
	languages["x-default"] = localePath(routing.defaultLocale, path);

	const fullTitle = absoluteTitle ? title : `${title} · ${siteName}`;

	return {
		title: absoluteTitle ? { absolute: title } : title,
		description,
		alternates: { canonical: url, languages },
		robots: index ? undefined : { index: false },
		openGraph: {
			type: "website",
			siteName,
			title: fullTitle,
			description: shareDescription,
			url,
			images: [shareImage],
			locale: openGraphLocales[locale],
			alternateLocale: routing.locales
				.filter((entry) => entry !== locale)
				.map((entry) => openGraphLocales[entry]),
		},
		twitter: {
			card: "summary_large_image",
			title: fullTitle,
			description: shareDescription,
			images: [
				{
					url: "/twitter-image.jpg",
					width: shareImage.width,
					height: shareImage.height,
					alt: shareImage.alt,
				},
			],
		},
	};
}

/** The absolute address of a page in a locale. */
export function absoluteUrl(locale: Locale, path = ""): string {
	const relative = localePath(locale, path);
	return relative === "/" ? siteUrl : `${siteUrl}${relative}`;
}
