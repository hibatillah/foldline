import type { Metadata } from "next";
import { hasLocale } from "next-intl";
import { getTranslations, setRequestLocale } from "next-intl/server";
import { notFound } from "next/navigation";

import { Extend } from "@/components/landing/extend";
import { Faq } from "@/components/landing/faq";
import { Finale } from "@/components/landing/finale";
import { Hero } from "@/components/landing/hero";
import { Light } from "@/components/landing/light";
import { PagesGrid } from "@/components/landing/pages-grid";
import { Proof } from "@/components/landing/proof";
import {
	SectionKeep,
	SectionNotice,
	SectionOpen,
	SectionPlan,
} from "@/components/landing/sections";
import { SiteFooter } from "@/components/landing/site-footer";
import { SiteNav } from "@/components/landing/site-nav";
import { Strip } from "@/components/landing/strip";
import { LANDING } from "@/components/landing/landing-classes";
import { JsonLd } from "@/components/landing/json-ld";
import { routing } from "@/i18n/routing";
import { pageMetadata } from "@/lib/seo";
import { structuredData } from "@/lib/structured-data";

type LocaleParams = { params: Promise<{ locale: string }> };

export async function generateMetadata({ params }: LocaleParams): Promise<Metadata> {
	const { locale } = await params;
	if (!hasLocale(routing.locales, locale)) return {};
	const t = await getTranslations({ locale });
	return pageMetadata({
		locale,
		title: t("meta.title"),
		description: t("meta.description"),
		absoluteTitle: true,
	});
}

export default async function Home({ params }: LocaleParams) {
	const { locale } = await params;
	if (!hasLocale(routing.locales, locale)) notFound();
	setRequestLocale(locale);
	const t = await getTranslations({ locale });
	const faq = t.raw("faq") as { q: string; a: string }[];

	return (
		<div className={LANDING}>
			<JsonLd data={structuredData({ locale, description: t("meta.description"), faq })} />
			<SiteNav />
			<main id="top">
				<Hero />
				<Proof />
				<Strip />
				<SectionOpen />
				<SectionNotice />
				<SectionPlan />
				<SectionKeep />
				<PagesGrid />
				<Light />
				<Extend />
				<Faq />
				<Finale />
			</main>
			<SiteFooter />
		</div>
	);
}
