/** The Windows 11 logo: four flat squares, no skew. */
export const windowsLogoPath =
	"M3 3h8.5v8.5H3V3Zm9.5 0H21v8.5h-8.5V3ZM3 12.5h8.5V21H3v-8.5Zm9.5 0H21V21h-8.5v-8.5Z";

/** The Windows logo, drawn in the text color. */
export function WindowsIcon() {
	return (
		<svg aria-hidden="true" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
			<path d={windowsLogoPath} />
		</svg>
	);
}
