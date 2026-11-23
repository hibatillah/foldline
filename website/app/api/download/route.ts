import { NextResponse } from "next/server";

const LATEST_RELEASE_API = "https://api.github.com/repos/hibatillah/foldline/releases/latest";
const RELEASES_PAGE = "https://github.com/hibatillah/foldline/releases/latest";

// GitHub answers unauthenticated callers 60 times an hour per address, so the answer is kept for five minutes.
export const revalidate = 300;

type ReleaseAsset = { name: string; browser_download_url: string };

/** Sends the visitor to the installer of the latest release; the releases page when it cannot be found. */
export async function GET() {
	try {
		const response = await fetch(LATEST_RELEASE_API, {
			headers: { Accept: "application/vnd.github+json", "User-Agent": "foldline-website" },
		});
		if (response.ok) {
			const release = (await response.json()) as { assets?: ReleaseAsset[] };
			const installer = release.assets?.find((asset) => asset.name.endsWith("-setup.exe"));
			if (installer) return NextResponse.redirect(installer.browser_download_url, 302);
		}
	} catch {
		// A failed lookup is not an error for the visitor; the releases page below still works.
	}
	return NextResponse.redirect(RELEASES_PAGE, 302);
}
