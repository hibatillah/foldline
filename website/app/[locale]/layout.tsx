import type { Metadata } from "next";
import { hasLocale, NextIntlClientProvider } from "next-intl";
import { getTranslations, setRequestLocale } from "next-intl/server";
import { Geist, Geist_Mono } from "next/font/google";
import { notFound } from "next/navigation";

import "../globals.css";
import { ThemeProvider } from "@/components/theme-provider";
import { routing } from "@/i18n/routing";
import { cn } from "@/lib/utils";

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
		title: t("meta.title"),
		description: t("meta.description"),
		alternates: { languages: { en: "/", id: "/id" } },
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
			className={cn("antialiased", fontMono.variable, "font-sans", geist.variable)}
		>
			<body>
				{/* The design has no dark version, so the page is light only. */}
				<ThemeProvider forcedTheme="light">
					<NextIntlClientProvider>{children}</NextIntlClientProvider>
				</ThemeProvider>
			</body>
		</html>
	);
}
