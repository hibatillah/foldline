import { defineRouting } from "next-intl/routing";

export const routing = defineRouting({
	locales: ["en", "id"],
	defaultLocale: "en",
	// English stays at "/", Indonesian lives at "/id".
	localePrefix: "as-needed",
	// The browser language and a saved choice are ignored: "/" is always English until a visitor switches.
	localeDetection: false,
	// No locale cookie, because the address alone decides the language.
	localeCookie: false,
});
