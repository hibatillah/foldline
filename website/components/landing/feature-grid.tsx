import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import type { NotchKey } from "@/components/notch/sizes";

import type { FeatureKey } from "./feature-head";
import { LightCard } from "./light-card";
import { NotchFrame } from "./notch-frame";

export type FeatureItem = {
	key: string;
	span: 1 | 2 | 3 | 4;
	kind: "frame" | "card";
	visual: ReactNode;
	zoom?: number;
	notch?: NotchKey;
	seed?: number;
};

type Caption = Record<string, { t: string; d: string }>;

/** A four-column grid of figures; each has a drawing and a caption read from `g.<section>`. */
export function FeatureGrid({ section, items }: { section: FeatureKey; items: FeatureItem[] }) {
	const t = useTranslations();
	const captions = t.raw(`g.${section}`) as Caption;

	return (
		<div className="g4">
			{items.map((item) => (
				<figure
					key={item.key}
					className={`s${item.span}`}
					style={{ margin: 0, display: "flex", flexDirection: "column", gap: 16, minWidth: 0 }}
				>
					{item.kind === "frame" && item.notch ? (
						<NotchFrame seed={item.seed ?? 1} zoom={item.zoom ?? 1} notch={item.notch}>
							{item.visual}
						</NotchFrame>
					) : (
						<LightCard zoom={item.zoom}>{item.visual}</LightCard>
					)}
					<figcaption style={{ display: "flex", flexDirection: "column", gap: 4 }}>
						<span style={{ fontSize: 17, fontWeight: 500 }}>{captions[item.key].t}</span>
						<span style={{ fontSize: 15, lineHeight: 1.55, color: "var(--muted-foreground)" }}>
							{captions[item.key].d}
						</span>
					</figcaption>
				</figure>
			))}
		</div>
	);
}
