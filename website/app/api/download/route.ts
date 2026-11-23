import { NextResponse } from "next/server";

const LATEST_RELEASE_API = "https://api.github.com/repos/hibatillah/foldline/releases/latest";
const RELEASES_PAGE = "https://github.com/hibatillah/foldline/releases/latest";

// Rendered on each request and cached once, at the CDN: a build-time copy or a second cache layer would keep serving the old release.
export const dynamic = "force-dynamic";

// GitHub answers unauthenticated callers 60 times an hour per address, so the CDN keeps the answer for five minutes.
const CACHE_CONTROL = "public, s-maxage=300, stale-while-revalidate=60";

type ReleaseAsset = { name: string; browser_download_url: string };

/** A redirect the CDN may cache for a few minutes. */
function cachedRedirect(url: string) {
	const response = NextResponse.redirect(url, 302);
	response.headers.set("Cache-Control", CACHE_CONTROL);
	return response;
}

/** Sends the visitor to the installer of the latest release; the releases page when it cannot be found. */
export async function GET() {
	try {
		const response = await fetch(LATEST_RELEASE_API, {
			headers: { Accept: "application/vnd.github+json", "User-Agent": "foldline-website" },
			cache: "no-store",
		});
		if (response.ok) {
			const release = (await response.json()) as { assets?: ReleaseAsset[] };
			const installer = release.assets?.find((asset) => asset.name.endsWith("-setup.exe"));
			if (installer) return cachedRedirect(installer.browser_download_url);
		}
	} catch {
		// A failed lookup is not an error for the visitor; the releases page below still works.
	}
	return cachedRedirect(RELEASES_PAGE);
}
