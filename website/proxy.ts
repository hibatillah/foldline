import createMiddleware from "next-intl/middleware";

import { routing } from "./i18n/routing";

export default createMiddleware(routing);

export const config = {
	// Skips the API, the PostHog proxy (/ingest), Next internals and files with an extension.
	matcher: "/((?!api|ingest|_next|_vercel|.*[.].*).*)",
};
