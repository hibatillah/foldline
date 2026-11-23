import type { Metadata } from "next";
import { hasLocale } from "next-intl";
import { getTranslations, setRequestLocale } from "next-intl/server";
import { notFound } from "next/navigation";

import { DownloadPage } from "@/components/landing/download-page";
import { routing } from "@/i18n/routing";
import { firstSentence, pageMetadata } from "@/lib/seo";

type LocaleParams = { params: Promise<{ locale: string }> };

export async function generateMetadata({ params }: LocaleParams): Promise<Metadata> {
	const { locale } = await params;
	if (!hasLocale(routing.locales, locale)) return {};
	const t = await getTranslations({ locale });
	// A page that only starts a download has no place in search results.
	return pageMetadata({
		locale,
		path: "/download",
		title: t("download.title"),
		description: firstSentence(t("download.subtitle")),
		index: false,
	});
}

export default async function Download({ params }: LocaleParams) {
	const { locale } = await params;
	if (!hasLocale(routing.locales, locale)) notFound();
	setRequestLocale(locale);

	return <DownloadPage />;
}
