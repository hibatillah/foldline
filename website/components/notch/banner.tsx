import type { ReactNode } from "react";

import { Flares } from "./open-notch";

/** A notification card that drops out of the pill. */
export function Banner({ width, children }: { width: number; children: ReactNode }) {
	return (
		<div
			style={{
				position: "relative",
				width,
				background: "#000",
				color: "#fff",
				borderRadius: "0 0 18px 18px",
				padding: "12px 20px 16px",
				boxSizing: "border-box",
				fontFamily: "var(--font-sans)",
				textAlign: "left",
			}}
		>
			<Flares size={6} />
			{children}
		</div>
	);
}
