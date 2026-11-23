import type { ReactNode } from "react";

import { buttonVariants } from "@/components/ui/button";
import { Link } from "@/i18n/navigation";
import { cn } from "@/lib/utils";

import { ExternalLink } from "./external-link";

const SIZE = {
	sm: "h-[38px]! gap-2! rounded-lg! px-4! text-sm!",
	lg: "h-11! gap-2! rounded-lg! px-5! text-[15px]!",
} as const;

const VARIANT = {
	primary: "",
	light: "relative bg-white! text-[#0b1a33]! hover:bg-white/90!",
	outline:
		"border-[color-mix(in_oklch,var(--foreground)_14%,transparent)]! bg-white! text-foreground! hover:bg-muted!",
	lightOutline: "relative border-white/45! bg-white/10! text-white! hover:bg-white/20!",
} as const;

/** A call-to-action link drawn as the site Button. */
export function CtaLink({
	href,
	size,
	variant,
	children,
}: {
	href: string;
	size: keyof typeof SIZE;
	variant: keyof typeof VARIANT;
	children: ReactNode;
}) {
	const className = cn(
		buttonVariants({ variant: "default" }),
		SIZE[size],
		VARIANT[variant],
		"no-underline",
	);
	// The download route leaves the site for GitHub, so it is tracked like an outside link.
	if (href.startsWith("http") || href.startsWith("/api/")) {
		return (
			<ExternalLink href={href} className={className} data-slot="button">
				{children}
			</ExternalLink>
		);
	}
	// A page of this site keeps the language the visitor chose.
	if (href.startsWith("/")) {
		return (
			<Link href={href} className={className} data-slot="button">
				{children}
			</Link>
		);
	}
	return (
		<a href={href} className={className} data-slot="button">
			{children}
		</a>
	);
}
