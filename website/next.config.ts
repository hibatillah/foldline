import type { NextConfig } from "next";
import createNextIntlPlugin from "next-intl/plugin";

import { assetsHost, ingestHost } from "./lib/posthog-hosts";

const withNextIntl = createNextIntlPlugin();

const nextConfig: NextConfig = {
	// Sends PostHog traffic through this site so blockers that match posthog.com do not drop it.
	async rewrites() {
		return [
			{ source: "/ingest/static/:path*", destination: `${assetsHost}/static/:path*` },
			{ source: "/ingest/array/:path*", destination: `${assetsHost}/array/:path*` },
			{ source: "/ingest/:path*", destination: `${ingestHost}/:path*` },
		];
	},
	// PostHog's endpoints end in a slash; do not redirect them.
	skipTrailingSlashRedirect: true,
};

export default withNextIntl(nextConfig);
