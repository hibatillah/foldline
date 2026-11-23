/** The facts of each published release that do not change with the language; the text is in `messages` under `releases.entries`, in the same order. */
export const RELEASES = [
	{
		version: "0.6.0",
		date: "2026-11-23",
		bytes: 6040572,
		sha256: "dc11e725f375be59ce064f3634501c59641767e62469e1a46f059c788afcd490",
	},
] as const;

const REPOSITORY = "https://github.com/hibatillah/foldline";

/** The GitHub address of the installer of a version. */
export function installerUrl(version: string): string {
	return `${REPOSITORY}/releases/download/v${version}/foldline-v${version}-windows-x64-setup.exe`;
}

/** The GitHub page of a release. */
export function releaseUrl(version: string): string {
	return `${REPOSITORY}/releases/tag/v${version}`;
}
