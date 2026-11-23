import { useTranslations } from "next-intl";
import type en from "@/messages/en.json";

type Messages = typeof en;

export function Faq() {
	const t = useTranslations();
	const faq = t.raw("faq") as Messages["faq"];

	return (
		<section id="faq" className="wrap" style={{ paddingTop: "clamp(64px, 7vw, 96px)" }}>
			<div style={{ marginBottom: "40px" }}>
				<h2
					className="h2"
					style={{ margin: "0", fontWeight: "500", letterSpacing: "-0.025em", textWrap: "balance" }}
				>
					{t("faqTitle.a")}
					<br />
					<span style={{ color: "var(--muted-foreground)" }}>{t("faqTitle.b")}</span>
				</h2>
			</div>
			<div className="faq">
				{faq.map((q) => (
					<details style={{ borderTop: "1px solid var(--border)", padding: "20px 0" }} key={q.q}>
						<summary
							style={{
								display: "flex",
								justifyContent: "space-between",
								gap: "16px",
								fontSize: "17px",
								fontWeight: "500",
							}}
						>
							{q.q}
							<svg
								className="plus"
								aria-hidden="true"
								width="18"
								height="18"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								strokeWidth="2"
								strokeLinecap="round"
								style={{ flex: "none", marginTop: "3px" }}
							>
								<path d="M12 5v14M5 12h14" />
							</svg>
						</summary>
						<p
							style={{
								margin: "12px 0 0",
								fontSize: "16px",
								lineHeight: "1.6",
								color: "var(--muted-foreground)",
								maxWidth: "60ch",
							}}
						>
							{q.a}
						</p>
					</details>
				))}
			</div>
		</section>
	);
}
