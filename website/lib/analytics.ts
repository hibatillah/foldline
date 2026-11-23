import posthog from "posthog-js";

/** Names of the events the site sends on purpose; autocapture covers every other click. */
export type SiteEvent = "download_clicked" | "github_clicked" | "docs_clicked";

/** Sends one named event; does nothing while analytics is off (no key). */
export function trackEvent(
	name: SiteEvent,
	properties?: Record<string, string | number | boolean>,
) {
	if (typeof window === "undefined" || !posthog.__loaded) return;
	posthog.capture(name, properties);
}
