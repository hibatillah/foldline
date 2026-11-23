/** Natural width in pixels of each notch drawing. */
export const NOTCH_WIDTH = {
	media: 614,
	calendar: 440,
	notification: 380,
	download: 340,
	weather: 526,
	prayer: 523,
	shelf: 500,
	clipboard: 540,
	countdown: 440,
} as const;

export type NotchKey = keyof typeof NOTCH_WIDTH;

/** The width class of each drawing; class names are written out whole so Tailwind finds them. */
export const NOTCH_WIDTH_CLASS: Record<NotchKey, string> = {
	media: "w-[614px]",
	calendar: "w-[440px]",
	notification: "w-[380px]",
	download: "w-[340px]",
	weather: "w-[526px]",
	prayer: "w-[523px]",
	shelf: "w-[500px]",
	clipboard: "w-[540px]",
	countdown: "w-[440px]",
};

/**
 * The zoom that fits each drawing into a phone: 342 px (a 390 px screen minus the page gutters) divided by its
 * width, never above 1. Written out as classes; keep them in step with NOTCH_WIDTH.
 */
export const PHONE_ZOOM_CLASS: Record<NotchKey, string> = {
	media: "max-[640px]:[zoom:0.557]!",
	calendar: "max-[640px]:[zoom:0.777]!",
	notification: "max-[640px]:[zoom:0.9]!",
	download: "max-[640px]:[zoom:1]!",
	weather: "max-[640px]:[zoom:0.65]!",
	prayer: "max-[640px]:[zoom:0.654]!",
	shelf: "max-[640px]:[zoom:0.684]!",
	clipboard: "max-[640px]:[zoom:0.633]!",
	countdown: "max-[640px]:[zoom:0.777]!",
};

/** The zoom classes a drawing may be given on a screen wider than a phone. */
export const ZOOM_CLASS: Record<number, string> = {
	0.82: "[zoom:0.82]",
	1: "[zoom:1]",
	1.05: "[zoom:1.05]",
	1.08: "[zoom:1.08]",
	1.1: "[zoom:1.1]",
	1.15: "[zoom:1.15]",
	1.2: "[zoom:1.2]",
	1.3: "[zoom:1.3]",
	1.35: "[zoom:1.35]",
	1.45: "[zoom:1.45]",
	1.5: "[zoom:1.5]",
};
