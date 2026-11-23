import posthog from "posthog-js";

import { appHost } from "./lib/posthog-hosts";

const apiKey = process.env.NEXT_PUBLIC_POSTHOG_KEY;

// Stays off until the key is set, so a build without it sends nothing.
if (apiKey) {
	posthog.init(apiKey, {
		api_host: "/ingest",
		ui_host: appHost,
		// Web analytics: a pageview on every route change, and the time spent on the page.
		capture_pageview: "history_change",
		capture_pageleave: true,
		// Product analytics: clicks on buttons and links are captured without extra code.
		autocapture: true,
		// Heatmaps and scroll depth.
		capture_heatmaps: true,
		// Errors and Core Web Vitals.
		capture_exceptions: true,
		capture_performance: true,
		// PostHog advises no person profiles in cookieless "always" mode; the site never identifies anyone.
		person_profiles: "never",
		// Nothing is stored on the device: PostHog counts a visit with a hash it makes on its servers, so no consent
		// banner is needed. Returning visitors are not recognised across days.
		cookieless_mode: "always",
		// Session replay is off on purpose.
		disable_session_recording: true,
	});
}
