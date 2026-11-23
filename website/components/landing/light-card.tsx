import type { ReactNode } from "react";

import { ZOOM_CLASS } from "@/components/notch/sizes";

/** A pale card that holds a drawing made outside the notch. */
export function LightCard({ zoom = 1.15, children }: { zoom?: number; children: ReactNode }) {
	return (
		<div className="box-border flex min-h-[260px] flex-1 items-center justify-center overflow-hidden rounded-[14px] bg-[oklch(0.965_0.006_250)] p-7 font-sans [box-shadow:inset_0_0_0_1px_color-mix(in_oklch,var(--foreground)_9%,transparent)] max-[640px]:min-h-0 max-[640px]:px-5 max-[640px]:py-8">
			<div className={`flex w-full justify-center ${ZOOM_CLASS[zoom]} max-[900px]:[zoom:1]!`}>
				{children}
			</div>
		</div>
	);
}
