"use client";

import { useEffect } from "react";

import { trackEvent } from "@/lib/analytics";

const DOWNLOAD_ROUTE = "/api/download";
const START_DELAY_MILLISECONDS = 1200;

/** Starts the installer download a moment after the page opens, so the warnings are read first. */
export function AutoDownload() {
	useEffect(() => {
		const timer = window.setTimeout(() => {
			trackEvent("download_clicked", { channel: "github" });
			window.location.assign(DOWNLOAD_ROUTE);
		}, START_DELAY_MILLISECONDS);
		return () => window.clearTimeout(timer);
	}, []);

	return null;
}
