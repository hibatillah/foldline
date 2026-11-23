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
} as const;

export type NotchKey = keyof typeof NOTCH_WIDTH;

/** Widest a drawing may be on a phone (390 px screen minus the page gutters). */
const PHONE_CONTENT_WIDTH = 342;

/** Zoom that fits a drawing into a phone's width; never enlarges it. */
export function phoneZoom(notch: NotchKey): number {
	return Math.min(1, PHONE_CONTENT_WIDTH / NOTCH_WIDTH[notch]);
}
