import type { MetadataRoute } from "next";

import { siteUrl } from "@/lib/site";

/** Search and AI crawlers may read every page; only the API and the analytics proxy are closed. */
export default function robots(): MetadataRoute.Robots {
	return {
		rules: { userAgent: "*", allow: "/", disallow: ["/api/", "/ingest/"] },
		sitemap: `${siteUrl}/sitemap.xml`,
	};
}
