import type { ReactNode } from "react";

/** A pale card that holds a drawing made outside the notch. */
export function LightCard({ zoom = 1.15, children }: { zoom?: number; children: ReactNode }) {
	return (
		<div
			className="lc"
			style={{
				flex: 1,
				minHeight: 260,
				borderRadius: 14,
				background: "oklch(0.965 0.006 250)",
				boxShadow: "inset 0 0 0 1px color-mix(in oklch, var(--foreground) 9%, transparent)",
				display: "flex",
				alignItems: "center",
				justifyContent: "center",
				padding: 28,
				boxSizing: "border-box",
				overflow: "hidden",
				fontFamily: "var(--font-sans)",
			}}
		>
			<div
				className="pz"
				style={{ zoom, display: "flex", justifyContent: "center", width: "100%" }}
			>
				{children}
			</div>
		</div>
	);
}
