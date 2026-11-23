"use client";

import { useLocale, useTranslations } from "next-intl";

import { Link, usePathname } from "@/i18n/navigation";

const LINK =
	"flex min-h-[26px] items-center rounded-md px-[9px] text-xs font-medium no-underline max-[640px]:px-[7px]";
const CHOSEN = "bg-card text-foreground [box-shadow:0_1px_2px_rgba(15,23,42,0.12)]";
const IDLE = "bg-transparent text-muted-foreground shadow-none";

/** Two links that swap the page between English and Indonesian. */
export function LanguageSwitch() {
	const t = useTranslations();
	const activeLocale = useLocale();
	const pathname = usePathname();

	return (
		<fieldset
			aria-label={t("langLabel")}
			className="m-0 flex min-w-0 rounded-lg border-0 bg-muted p-0.5"
		>
			{(["en", "id"] as const).map((locale) => (
				<Link
					key={locale}
					href={pathname}
					locale={locale}
					scroll={false}
					hrefLang={locale}
					aria-current={activeLocale === locale ? "true" : undefined}
					// A named slot keeps the page's hover color off the chosen language.
					data-slot="language"
					className={`${LINK} ${activeLocale === locale ? CHOSEN : IDLE}`}
				>
					{locale.toUpperCase()}
				</Link>
			))}
		</fieldset>
	);
}
