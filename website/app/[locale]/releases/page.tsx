import type { Metadata } from "next";
import { hasLocale } from "next-intl";
import { getTranslations, setRequestLocale } from "next-intl/server";
import { notFound } from "next/navigation";

import { ReleasesPage } from "@/components/landing/releases-page";
import { routing } from "@/i18n/routing";
import { pageMetadata } from "@/lib/seo";

type LocaleParams = { params: Promise<{ locale: string }> };

export async function generateMetadata({ params }: LocaleParams): Promise<Metadata> {
	const { locale } = await params;
	if (!hasLocale(routing.locales, locale)) return {};
	const t = await getTranslations({ locale });
	return pageMetadata({
		locale,
		path: "/releases",
		title: t("releases.title"),
		description: t("releases.intro"),
	});
}

export default async function Releases({ params }: LocaleParams) {
	const { locale } = await params;
	if (!hasLocale(routing.locales, locale)) notFound();
	setRequestLocale(locale);

	return <ReleasesPage />;
}
