import { useTranslations } from "next-intl";

import type en from "@/messages/en.json";
import { Link } from "@/i18n/navigation";

import { PageShell } from "./page-shell";

type LegalDocument = Messages["privacy"];
type Messages = typeof en;

const LINK_PATTERN = /(\[[^\]]+\]\([^)]+\))/g;
const LINK_PARTS = /^\[([^\]]+)\]\(([^)]+)\)$/;

/** Text where [label](url) becomes an outside link. */
function RichText({ text }: { text: string }) {
	return text.split(LINK_PATTERN).map((part) => {
		const link = LINK_PARTS.exec(part);
		if (!link) return part;
		return (
			<a
				key={part}
				href={link[2]}
				target="_blank"
				rel="noreferrer"
				style={{ color: "var(--primary)" }}
			>
				{link[1]}
			</a>
		);
	});
}

/** A policy page: a slim header, the title and dated intro, numbered sections and the site footer. */
export function LegalPage({ document }: { document: "privacy" | "terms" }) {
	const t = useTranslations();
	const content = t.raw(document) as LegalDocument;

	return (
		<PageShell>
			<article style={{ maxWidth: "720px" }}>
				<Link
					href="/"
					style={{
						fontSize: "14px",
						color: "var(--muted-foreground)",
						textDecoration: "none",
					}}
				>
					{t("legalHome")}
				</Link>
				<h1 className="h1" style={{ margin: "16px 0 8px", fontWeight: "600" }}>
					{content.title}
				</h1>
				<p style={{ margin: "0 0 24px", fontSize: "14px", color: "var(--muted-foreground)" }}>
					{t("legalUpdated")}: {t("legalUpdatedOn")}
				</p>
				<p style={{ margin: "0 0 40px", fontSize: "17px", lineHeight: "1.6" }}>{content.intro}</p>
				{content.sections.map((section, index) => (
					<section key={section.h} style={{ marginBottom: "36px" }}>
						<h2 className="h3" style={{ margin: "0 0 12px", fontWeight: "600", fontSize: "20px" }}>
							{index + 1}. {section.h}
						</h2>
						{section.p.map((paragraph) => (
							<p
								key={paragraph}
								style={{
									margin: "0 0 12px",
									fontSize: "16px",
									lineHeight: "1.65",
									color: "var(--muted-foreground)",
								}}
							>
								<RichText text={paragraph} />
							</p>
						))}
						{"ul" in section && section.ul && (
							<ul
								style={{
									margin: "0 0 12px",
									paddingLeft: "22px",
									fontSize: "16px",
									lineHeight: "1.65",
									color: "var(--muted-foreground)",
								}}
							>
								{section.ul.map((item) => (
									<li key={item} style={{ marginBottom: "6px" }}>
										<RichText text={item} />
									</li>
								))}
							</ul>
						)}
					</section>
				))}
			</article>
		</PageShell>
	);
}
