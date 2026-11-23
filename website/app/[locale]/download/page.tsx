import type { Metadata } from "next";
import { hasLocale } from "next-intl";
import { getTranslations, setRequestLocale } from "next-intl/server";
import { notFound } from "next/navigation";

import { DownloadPage } from "@/components/landing/download-page";
import { routing } from "@/i18n/routing";

import "../landing.css";

type LocaleParams = { params: Promise<{ locale: string }> };

export async function generateMetadata({ params }: LocaleParams): Promise<Metadata> {
	const { locale } = await params;
	if (!hasLocale(routing.locales, locale)) return {};
	const t = await getTranslations({ locale });
	// A page that only starts a download has no place in search results.
	return { title: `${t("download.title")} · Foldline`, robots: { index: false } };
}

export default async function Download({ params }: LocaleParams) {
	const { locale } = await params;
	if (!hasLocale(routing.locales, locale)) notFound();
	setRequestLocale(locale);

	return <DownloadPage />;
}
