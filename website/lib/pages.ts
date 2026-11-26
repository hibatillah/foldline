/** The ten pages in the order of the pages grid; the key names the picture in `public/pages` and the route `/pages/<key>`. */
export const PAGE_KEYS = [
	"media",
	"shelf",
	"calendar",
	"timer",
	"todo",
	"prayer",
	"weather",
	"stats",
	"clipboard",
	"devices",
] as const;

export type PageKey = (typeof PAGE_KEYS)[number];

/** The pictures of `public/pages`, taken from the final designs on a dither background; each is as wide as its notch plus a margin. */
export const PAGE_PICTURE: Record<PageKey, { width: number; height: number }> = {
	media: { width: 734, height: 184 },
	shelf: { width: 660, height: 184 },
	calendar: { width: 660, height: 250 },
	timer: { width: 442, height: 184 },
	todo: { width: 660, height: 242 },
	prayer: { width: 660, height: 184 },
	weather: { width: 660, height: 184 },
	stats: { width: 844, height: 184 },
	clipboard: { width: 660, height: 277 },
	devices: { width: 592, height: 184 },
};

export function isPageKey(value: string): value is PageKey {
	return (PAGE_KEYS as readonly string[]).includes(value);
}
