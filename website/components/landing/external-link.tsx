import type { ComponentProps } from "react";

import type { SiteEvent } from "@/lib/analytics";

import { TrackedLink } from "./tracked-link";

/** Picks the analytics event that matches where a link goes. */
function eventFor(href: string): { event: SiteEvent; properties?: Record<string, string> } | null {
	if (href.startsWith("/api/download"))
		return { event: "download_clicked", properties: { channel: "github" } };
	if (href.includes("performance-baseline")) return { event: "docs_clicked" };
	if (href.includes("github.com")) return { event: "github_clicked" };
	return null;
}

/** A link that leaves the site; it reports its click when it matches a known destination. */
export function ExternalLink({ href, ...anchor }: ComponentProps<"a"> & { href: string }) {
	const tracked = eventFor(href);
	return (
		<TrackedLink href={href} event={tracked?.event} properties={tracked?.properties} {...anchor} />
	);
}
