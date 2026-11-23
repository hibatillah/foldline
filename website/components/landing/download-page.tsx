import { useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";
import type en from "@/messages/en.json";

import { AutoDownload } from "./auto-download";
import { CtaLink } from "./cta-link";
import { PageShell } from "./page-shell";
import { WindowsIcon } from "./windows-icon";

type Messages = typeof en;

const REPOSITORY = "https://github.com/hibatillah/foldline";

const BODY_STYLE = {
	margin: "0 0 12px",
	fontSize: "16px",
	lineHeight: 1.65,
	color: "var(--muted-foreground)",
} as const;

const HEADING_STYLE = { margin: "0 0 12px", fontWeight: 600, fontSize: "20px" } as const;

/** The page every Download button opens: it starts the download and explains the warnings that follow. */
export function DownloadPage() {
	const t = useTranslations();
	const content = t.raw("download") as Messages["download"];

	return (
		<PageShell>
			<AutoDownload />
			<article style={{ maxWidth: "720px" }}>
				<h1 className="h1" style={{ margin: "0 0 8px", fontWeight: 600 }}>
					{content.title}
				</h1>
				<p style={{ margin: "0 0 24px", fontSize: "17px", lineHeight: 1.6 }}>{content.subtitle}</p>
				<div style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: "16px" }}>
					<CtaLink href="/api/download" size="lg" variant="primary">
						<WindowsIcon />
						{content.again}
					</CtaLink>
					<span style={{ fontSize: "14px", color: "var(--muted-foreground)" }}>{content.meta}</span>
				</div>

				<h2 style={{ ...HEADING_STYLE, margin: "48px 0 8px" }}>{content.stepsTitle}</h2>
				<p style={BODY_STYLE}>{content.stepsIntro}</p>
				<ol
					style={{
						listStyle: "none",
						margin: "16px 0 0",
						padding: 0,
						display: "grid",
						gap: "20px",
					}}
				>
					{content.steps.map((step, index) => (
						<li key={step.h} style={{ display: "flex", gap: "16px" }}>
							<span
								aria-hidden="true"
								style={{
									flex: "none",
									width: "28px",
									height: "28px",
									borderRadius: "50%",
									display: "grid",
									placeItems: "center",
									fontSize: "14px",
									fontWeight: 600,
									background: "var(--primary)",
									color: "var(--primary-foreground)",
								}}
							>
								{index + 1}
							</span>
							<div>
								<h3 style={{ margin: "2px 0 6px", fontSize: "16px", fontWeight: 600 }}>{step.h}</h3>
								<p style={{ ...BODY_STYLE, margin: 0 }}>{step.p}</p>
							</div>
						</li>
					))}
				</ol>

				<h2 style={{ ...HEADING_STYLE, margin: "48px 0 8px" }}>{content.whyTitle}</h2>
				<p style={BODY_STYLE}>{content.why}</p>

				<h2 style={{ ...HEADING_STYLE, margin: "48px 0 8px" }}>{content.safeTitle}</h2>
				{content.safe.map((paragraph) => (
					<p key={paragraph} style={BODY_STYLE}>
						{paragraph}
					</p>
				))}
				<div style={{ display: "flex", flexWrap: "wrap", gap: "12px", marginTop: "20px" }}>
					<CtaLink href={REPOSITORY} size="lg" variant="outline">
						{content.source}
					</CtaLink>
					<Link
						href="/privacy"
						style={{
							display: "inline-flex",
							alignItems: "center",
							height: "44px",
							padding: "0 4px",
							fontSize: "15px",
							color: "var(--primary)",
						}}
					>
						{content.privacy}
					</Link>
				</div>
			</article>
		</PageShell>
	);
}
