import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { NotchKey } from "@/components/notch/sizes";

import type { FeatureKey } from "./feature-head";
import { LightCard } from "./light-card";
import { NotchFrame } from "./notch-frame";
import type { TextureKey } from "./textures";

export type FeatureItem = {
	key: string;
	span: 1 | 2 | 3 | 4;
	kind: "frame" | "card";
	visual: ReactNode;
	zoom?: number;
	notch?: NotchKey;
	seed?: number;
	/** The dithered picture behind a frame, from `public/textures`. */
	texture?: TextureKey;
};

/** How many of the four columns a figure takes: all four narrow to two on a tablet and one on a phone. */
const SPAN = {
	1: "col-span-1",
	2: "col-span-2 max-[640px]:col-span-1",
	3: "col-span-3 max-[900px]:col-span-2 max-[640px]:col-span-1",
	4: "col-span-4 max-[900px]:col-span-2 max-[640px]:col-span-1",
} as const;

type Caption = Record<string, { t: string; d: string }>;

/** A four-column grid of figures; each has a drawing and a caption read from `g.<section>`. */
export function FeatureGrid({ section, items }: { section: FeatureKey; items: FeatureItem[] }) {
	const t = useTranslations();
	const captions = t.raw(`g.${section}`) as Caption;

	return (
		<div className="grid grid-cols-[repeat(4,minmax(0,1fr))] gap-x-6 gap-y-10 max-[900px]:grid-cols-[repeat(2,minmax(0,1fr))] max-[640px]:grid-cols-[minmax(0,1fr)] max-[640px]:gap-10 [&>figure]:grid [&>figure]:row-span-2 [&>figure]:grid-rows-subgrid [&>figure]:gap-y-4">
			{items.map((item) => (
				<figure key={item.key} className={`${SPAN[item.span]} m-0 min-w-0`}>
					{item.kind === "frame" && item.notch ? (
						<NotchFrame
							seed={item.seed ?? 1}
							zoom={item.zoom ?? 1}
							notch={item.notch}
							texture={item.texture}
						>
							{item.visual}
						</NotchFrame>
					) : (
						<LightCard zoom={item.zoom}>{item.visual}</LightCard>
					)}
					<figcaption className="flex flex-col gap-[4px]">
						<span className="text-[17px] font-medium">{captions[item.key].t}</span>
						<span className="text-[15px] leading-[1.55] text-muted-foreground">
							{captions[item.key].d}
						</span>
					</figcaption>
				</figure>
			))}
		</div>
	);
}
