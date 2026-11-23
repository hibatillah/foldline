"use client";

import { useLocale, useTranslations } from "next-intl";

import { Link, usePathname } from "@/i18n/navigation";

const CHOSEN = {
	background: "var(--card)",
	color: "var(--foreground)",
	boxShadow: "0 1px 2px rgba(15,23,42,0.12)",
};
const IDLE = { background: "transparent", color: "var(--muted-foreground)", boxShadow: "none" };

/** Two links that swap the page between English and Indonesian. */
export function LanguageSwitch() {
	const t = useTranslations();
	const activeLocale = useLocale();
	const pathname = usePathname();

	return (
		<fieldset
			aria-label={t("langLabel")}
			className="lang"
			style={{
				display: "flex",
				padding: 2,
				margin: 0,
				minWidth: 0,
				border: 0,
				borderRadius: 8,
				background: "var(--muted)",
			}}
		>
			{(["en", "id"] as const).map((locale) => (
				<Link
					key={locale}
					href={pathname}
					locale={locale}
					scroll={false}
					hrefLang={locale}
					aria-current={activeLocale === locale ? "true" : undefined}
					style={activeLocale === locale ? CHOSEN : IDLE}
				>
					{locale.toUpperCase()}
				</Link>
			))}
		</fieldset>
	);
}
