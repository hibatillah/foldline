"use client";

import type { ComponentProps } from "react";

import { trackEvent, type SiteEvent } from "@/lib/analytics";

type TrackedLinkProps = ComponentProps<"a"> & {
	event?: SiteEvent;
	properties?: Record<string, string | number | boolean>;
};

/** An anchor that sends one analytics event when it is clicked. */
export function TrackedLink({
	event,
	properties,
	onClick,
	href,
	children,
	...anchor
}: TrackedLinkProps) {
	return (
		<a
			{...anchor}
			href={href}
			onClick={(clickEvent) => {
				if (event) trackEvent(event, properties);
				onClick?.(clickEvent);
			}}
		>
			{children}
		</a>
	);
}
