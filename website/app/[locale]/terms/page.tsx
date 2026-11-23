import type { Metadata } from "next";
import { hasLocale } from "next-intl";
import { getTranslations, setRequestLocale } from "next-intl/server";
import { notFound } from "next/navigation";

import { LegalPage } from "@/components/landing/legal-page";
import { routing } from "@/i18n/routing";
import { firstSentence, pageMetadata } from "@/lib/seo";

type LocaleParams = { params: Promise<{ locale: string }> };

export async function generateMetadata({ params }: LocaleParams): Promise<Metadata> {
	const { locale } = await params;
	if (!hasLocale(routing.locales, locale)) return {};
	const t = await getTranslations({ locale });
	return pageMetadata({
		locale,
		path: "/terms",
		title: t("terms.title"),
		description: firstSentence(t("terms.intro")),
	});
}

export default async function TermsPage({ params }: LocaleParams) {
	const { locale } = await params;
	if (!hasLocale(routing.locales, locale)) notFound();
	setRequestLocale(locale);

	return <LegalPage document="terms" />;
}
