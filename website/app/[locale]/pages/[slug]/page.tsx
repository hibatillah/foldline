import type { Metadata } from "next";
import { hasLocale } from "next-intl";
import { getTranslations, setRequestLocale } from "next-intl/server";
import { notFound } from "next/navigation";

import { PageDetail } from "@/components/landing/page-detail";
import { routing } from "@/i18n/routing";
import { isPageKey, PAGE_KEYS } from "@/lib/pages";
import { pageMetadata } from "@/lib/seo";

type PageParams = { params: Promise<{ locale: string; slug: string }> };

export function generateStaticParams() {
	return routing.locales.flatMap((locale) => PAGE_KEYS.map((slug) => ({ locale, slug })));
}

export async function generateMetadata({ params }: PageParams): Promise<Metadata> {
	const { locale, slug } = await params;
	if (!hasLocale(routing.locales, locale) || !isPageKey(slug)) return {};
	const t = await getTranslations({ locale });
	return pageMetadata({
		locale,
		path: `/pages/${slug}`,
		title: t(`grid.${slug}`),
		description: t(`pageDetail.items.${slug}.lead`),
	});
}

export default async function PagesDetail({ params }: PageParams) {
	const { locale, slug } = await params;
	if (!hasLocale(routing.locales, locale) || !isPageKey(slug)) notFound();
	setRequestLocale(locale);

	return <PageDetail pageKey={slug} />;
}
