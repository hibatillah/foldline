import { hasLocale } from "next-intl";

import { routing } from "@/i18n/routing";
import { ogSize, renderOgImage } from "@/lib/og-image";

export function generateStaticParams() {
	return routing.locales.map((locale) => ({ locale }));
}

export const alt = "Foldline: a Dynamic Island notch for the top of your Windows screen";
export const size = ogSize;
export const contentType = "image/png";

export default async function Image({ params }: { params: Promise<{ locale: string }> }) {
	const { locale } = await params;
	return renderOgImage(hasLocale(routing.locales, locale) ? locale : routing.defaultLocale);
}
