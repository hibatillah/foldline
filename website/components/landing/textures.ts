/** The dithered pictures of `public/textures`, as background classes (a class name must appear whole in the source). */
export const TEXTURE = {
	universtock: "bg-[url(/textures/universtock-dither.png)]",
	henrikLight: "bg-[url(/textures/henrik-mceh-dither.png)]",
	henrikDark: "bg-[url(/textures/henrik-t2sai-dither.png)]",
	blueWide: "bg-[url(/textures/blue-dither-wide.png)]",
} as const;

export type TextureKey = keyof typeof TEXTURE;

/** How a picture fills a panel: covering it, over a deep blue that shows while it loads. */
export const TEXTURE_FILL = "bg-cover bg-no-repeat bg-[color:oklch(0.2_0.05_255)]";

/** Where the picture sits in a panel, so panels that share a picture do not show the same crop. */
export const TEXTURE_POSITIONS = [
	"bg-[position:20%_30%]",
	"bg-[position:80%_70%]",
	"bg-[position:50%_50%]",
	"bg-[position:10%_80%]",
] as const;

/** The shared look of a panel that holds a drawing: rounded, clipped, with a hairline inner ring. */
export const PANEL =
	"relative flex justify-center overflow-hidden rounded-[14px] [box-shadow:inset_0_0_0_1px_color-mix(in_oklch,var(--foreground)_7%,transparent)]";
