import type { ReactNode } from "react";

import { PHONE_ZOOM_CLASS, ZOOM_CLASS, type NotchKey } from "@/components/notch/sizes";

import { PANEL, TEXTURE, TEXTURE_FILL, TEXTURE_POSITIONS, type TextureKey } from "./textures";

/** The pictures handed out to the grid frames by seed. */
const GRID_TEXTURES: TextureKey[] = ["universtock", "henrikLight", "henrikDark"];

const VARIANT = {
	hero: "flex-1 box-border pb-[clamp(64px,7vw,104px)] max-[640px]:pb-10",
	grid: "flex-1 box-border min-h-[260px] pb-9 max-[640px]:min-h-0 max-[640px]:pb-6",
} as const;

/** The textured panel a notch drawing sits in; under 640 px the drawing's `zoom` becomes its own `--mz`. */
export function NotchFrame({
	seed,
	zoom,
	notch,
	variant = "grid",
	texture,
	children,
}: {
	seed: number;
	zoom: number;
	notch: NotchKey;
	variant?: "hero" | "grid";
	/** The background picture; a grid frame takes one by its seed when this is left out. */
	texture?: TextureKey;
	children: ReactNode;
}) {
	const picture = texture ?? GRID_TEXTURES[seed % GRID_TEXTURES.length];

	return (
		<div
			className={`${PANEL} ${TEXTURE[picture]} ${TEXTURE_FILL} ${TEXTURE_POSITIONS[seed % TEXTURE_POSITIONS.length]} ${VARIANT[variant]}`}
		>
			<div
				className={`relative ${ZOOM_CLASS[zoom]} max-[900px]:[zoom:0.75]! ${PHONE_ZOOM_CLASS[notch]}`}
			>
				{children}
			</div>
		</div>
	);
}
