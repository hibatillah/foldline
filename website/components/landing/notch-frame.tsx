import type { CSSProperties, ReactNode } from "react";

import { phoneZoom, type NotchKey } from "@/components/notch/sizes";

import { frameStyle } from "./frame-style";
import { Waves } from "./waves";

/** The tinted panel a notch drawing sits in; under 640 px the CSS swaps `zoom` for `--mz`. */
export function NotchFrame({
	seed,
	zoom,
	notch,
	variant = "grid",
	children,
}: {
	seed: number;
	zoom: number;
	notch: NotchKey;
	variant?: "hero" | "grid";
	children: ReactNode;
}) {
	const panel: CSSProperties =
		variant === "hero"
			? { ...frameStyle, height: "auto", paddingBottom: "clamp(40px, 4.5vw, 64px)" }
			: {
					...frameStyle,
					height: "auto",
					flex: 1,
					minHeight: 260,
					paddingBottom: 36,
					boxSizing: "border-box",
				};
	const drawing = {
		position: "relative",
		zoom,
		"--mz": phoneZoom(notch).toFixed(3),
	} as CSSProperties;

	return (
		<div className={variant === "grid" ? "nf" : undefined} style={panel}>
			<Waves seed={seed} />
			<div className="z zf" style={drawing}>
				{children}
			</div>
		</div>
	);
}
