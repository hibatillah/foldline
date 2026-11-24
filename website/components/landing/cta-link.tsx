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
	muted:
		"bg-[color-mix(in_oklch,var(--foreground)_9%,transparent)]! text-foreground! hover:bg-[color-mix(in_oklch,var(--foreground)_14%,transparent)]!",
	lightOutline:
		"relative border-white! bg-transparent! text-white! hover:bg-white/15! dark:bg-transparent! dark:hover:bg-white/15!",
	outlineOnly:
		"border-[color-mix(in_oklch,var(--foreground)_18%,transparent)]! bg-transparent! text-foreground! hover:bg-muted!",
} as const;

/** A call-to-action link drawn as the site Button. */
export function CtaLink({
	href,
	size,
	variant,
	className: extraClassName,
	children,
}: {
	href: string;
	size: keyof typeof SIZE;
	variant: keyof typeof VARIANT;
	className?: string;
	children: ReactNode;
}) {
	const className = cn(
		buttonVariants({ variant: "default" }),
		SIZE[size],
		VARIANT[variant],
		"no-underline",
		extraClassName,
	);
	// The download route leaves the site for GitHub, so it is tracked like an outside link.
	if (href.startsWith("http") || href.startsWith("/api/")) {
		return (
			<ExternalLink
				href={href}
				className={className}
				data-slot="button"
				{...(href.startsWith("http") ? { target: "_blank", rel: "noopener noreferrer" } : {})}
			>
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
