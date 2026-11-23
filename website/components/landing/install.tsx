import { useTranslations } from "next-intl";
import type en from "@/messages/en.json";
import { CtaLink } from "./cta-link";
import { Waves } from "./waves";
import { ReleaseCard } from "./release-card";

type Messages = typeof en;

export function Install() {
	const t = useTranslations();
	const installSteps = t.raw("install.steps") as Messages["install"]["steps"];

	return (
		<>
			<div className="wrap" style={{ marginTop: "clamp(120px, 13vw, 200px)" }}>
				<hr className="rule" />
			</div>
			<section
				id="install"
				className="wrap duo"
				style={{ padding: "clamp(64px, 7vw, 96px) clamp(16px, 5vw, 80px)", alignItems: "center" }}
			>
				<div
					style={{
						display: "flex",
						flexDirection: "column",
						gap: "36px",
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
						{t("install.a")}
						<br />
						<span style={{ color: "var(--muted-foreground)" }}>{t("install.b")}</span>
					</h2>
					<ol
						style={{
							margin: "0",
							padding: "0",
							listStyle: "none",
							display: "flex",
							flexDirection: "column",
							gap: "14px",
							fontSize: "16px",
							lineHeight: "1.5",
						}}
					>
						{installSteps.map((st) => (
							<li style={{ display: "flex", gap: "14px" }} key={st.n}>
								<span
									style={{
										color: "var(--muted-foreground)",
										fontVariantNumeric: "tabular-nums",
										width: "14px",
									}}
								>
									{st.n}
								</span>
								<span>{st.t}</span>
							</li>
						))}
					</ol>
					<CtaLink href="/download" size="lg" variant="primary">
						{t("hero.cta")}
					</CtaLink>
				</div>
				<div
					style={{
						position: "relative",
						overflow: "hidden",
						borderRadius: "14px",
						height: "auto",
						background:
							"radial-gradient(70% 90% at 80% 0%, oklch(0.93 0.035 250) 0%, transparent 70%), radial-gradient(60% 80% at 10% 100%, oklch(0.9 0.05 255) 0%, transparent 75%), oklch(0.965 0.01 250)",
						boxShadow: "inset 0 0 0 1px color-mix(in oklch, var(--foreground) 7%, transparent)",
						display: "flex",
						justifyContent: "center",
						padding: "48px 0",
					}}
				>
					<Waves seed={7} />
					<div
						style={{
							position: "relative",
							width: "100%",
							display: "flex",
							justifyContent: "center",
						}}
					>
						<ReleaseCard />
					</div>
				</div>
			</section>
			<div className="wrap">
				<hr className="rule" />
			</div>
		</>
	);
}
