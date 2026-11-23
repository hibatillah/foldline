import type { Metadata } from "next";
import { hasLocale, NextIntlClientProvider } from "next-intl";
import { getTranslations, setRequestLocale } from "next-intl/server";
import { Geist, Geist_Mono } from "next/font/google";
import { notFound } from "next/navigation";
import Script from "next/script";

import "../globals.css";
import { ThemeProvider } from "@/components/theme-provider";
import { routing } from "@/i18n/routing";
import { authorName, authorUrl, siteName, siteUrl } from "@/lib/site";
import { cn } from "@/lib/utils";

/** react-grab runs in development, and in a production build only when NEXT_PUBLIC_REACT_GRAB is "true" (read at build time). */
const loadReactGrab =
	process.env.NODE_ENV !== "production" || process.env.NEXT_PUBLIC_REACT_GRAB === "true";

const geist = Geist({ subsets: ["latin"], variable: "--font-sans" });

const fontMono = Geist_Mono({
	subsets: ["latin"],
	variable: "--font-mono",
});

type LocaleParams = { params: Promise<{ locale: string }> };

export function generateStaticParams() {
	return routing.locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }: LocaleParams): Promise<Metadata> {
	const { locale } = await params;
	if (!hasLocale(routing.locales, locale)) return {};
	const t = await getTranslations({ locale });

	return {
		metadataBase: new URL(siteUrl),
		// Pages pass a short title, and the template adds the site name.
		title: { default: t("meta.title"), template: `%s · ${siteName}` },
		description: t("meta.description"),
		applicationName: siteName,
		authors: [{ name: authorName, url: authorUrl }],
		creator: authorName,
		category: "utilities",
	};
}

export default async function RootLayout({
	children,
	params,
}: Readonly<{ children: React.ReactNode } & LocaleParams>) {
	const { locale } = await params;
	if (!hasLocale(routing.locales, locale)) notFound();
	setRequestLocale(locale);

	return (
		<html
			lang={locale}
			suppressHydrationWarning
			// Smooth in-page jumps; Next switches it off while it changes page, so a new page opens at once.
			data-scroll-behavior="smooth"
			className={cn(
				"scroll-smooth antialiased motion-reduce:scroll-auto",
				fontMono.variable,
				"font-sans",
				geist.variable,
			)}
		>
			<body>
				{/* The design has no dark version, so the page is light only. */}
				<ThemeProvider forcedTheme="light">
					<NextIntlClientProvider>{children}</NextIntlClientProvider>
				</ThemeProvider>
				{loadReactGrab && (
					<Script
						src="//unpkg.com/react-grab/dist/index.global.js"
						crossOrigin="anonymous"
						strategy="beforeInteractive"
					/>
				)}
			</body>
		</html>
	);
}
