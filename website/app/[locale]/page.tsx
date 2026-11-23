import { hasLocale } from "next-intl";
import { setRequestLocale } from "next-intl/server";
import { notFound } from "next/navigation";

import { Extend } from "@/components/landing/extend";
import { Faq } from "@/components/landing/faq";
import { Finale } from "@/components/landing/finale";
import { Hero } from "@/components/landing/hero";
import { Install } from "@/components/landing/install";
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
import { routing } from "@/i18n/routing";

import "./landing.css";

export default async function Home({ params }: { params: Promise<{ locale: string }> }) {
	const { locale } = await params;
	if (!hasLocale(routing.locales, locale)) notFound();
	setRequestLocale(locale);

	return (
		<div
			className="landing"
			style={{
				fontFamily: "var(--font-sans)",
				color: "var(--foreground)",
				background: "var(--background)",
				WebkitFontSmoothing: "antialiased",
			}}
		>
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
				<Install />
				<Faq />
				<Finale />
			</main>
			<SiteFooter />
		</div>
	);
}
