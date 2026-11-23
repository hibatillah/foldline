import type { CSSProperties } from "react";

/** The blue-tinted panel behind every notch drawing. */
export const frameStyle: CSSProperties = {
	position: "relative",
	overflow: "hidden",
	borderRadius: "14px",
	height: "auto",
	background:
		"radial-gradient(70% 90% at 80% 0%, oklch(0.93 0.035 250) 0%, transparent 70%), radial-gradient(60% 80% at 10% 100%, oklch(0.9 0.05 255) 0%, transparent 75%), oklch(0.965 0.01 250)",
	boxShadow: "inset 0 0 0 1px color-mix(in oklch, var(--foreground) 7%, transparent)",
	display: "flex",
	justifyContent: "center",
};
