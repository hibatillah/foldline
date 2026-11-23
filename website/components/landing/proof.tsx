import { useTranslations } from "next-intl";
import { ExternalLink } from "./external-link";

export function Proof() {
	const t = useTranslations();

	return (
		<>
			<div className="wrap" style={{ marginTop: "clamp(80px, 9vw, 120px)" }}>
				<hr className="rule" />
			</div>
			<section
				className="wrap duo"
				style={{ padding: "clamp(64px, 7vw, 96px) clamp(16px, 5vw, 80px)" }}
			>
				<div
					style={{
						display: "flex",
						flexDirection: "column",
						gap: "40px",
						alignItems: "flex-start",
					}}
				>
					<h2
						className="h2"
						style={{
							margin: "0",
							fontWeight: "500",
							letterSpacing: "-0.025em",
							textWrap: "balance",
						}}
					>
						{t("proof.a")}
						<br />
						<span style={{ color: "var(--muted-foreground)" }}>{t("proof.b")}</span>
					</h2>
					<ExternalLink
						href="https://github.com/hibatillah/foldline/blob/master/docs/reference/performance-baseline.md"
						style={{
							display: "flex",
							alignItems: "center",
							gap: "14px",
							textDecoration: "none",
							fontSize: "15px",
							fontWeight: "500",
						}}
					>
						<span
							style={{
								width: "40px",
								height: "40px",
								borderRadius: "50%",
								background: "var(--muted)",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
							}}
						>
							<svg
								aria-hidden="true"
								width="16"
								height="16"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								strokeWidth="2"
								strokeLinecap="round"
							>
								<path d="M5 12h14M13 6l6 6-6 6" />
							</svg>
						</span>
						{t("proof.link")}
					</ExternalLink>
				</div>
				<figure style={{ margin: "0", display: "flex", flexDirection: "column", gap: "28px" }}>
					<p
						style={{
							margin: "0",
							fontSize: "clamp(20px, 1.8vw, 24px)",
							lineHeight: "1.45",
							letterSpacing: "-0.01em",
						}}
					>
						{t("proof.quote")}
					</p>
					<figcaption style={{ fontSize: "14px", color: "var(--muted-foreground)" }}>
						{t("proof.source")}
					</figcaption>
				</figure>
			</section>
			<div className="wrap">
				<hr className="rule" />
			</div>
		</>
	);
}
