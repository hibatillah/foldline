import { useTranslations } from "next-intl";
import type en from "@/messages/en.json";
import { Waves } from "./waves";

type Messages = typeof en;

export function Extend() {
	const t = useTranslations();
	const ownSteps = t.raw("own.steps") as Messages["own"]["steps"];

	return (
		<section className="wrap feature">
			<div className="fhead">
				<p className="label" style={{ margin: "0" }}>
					<span style={{ color: "var(--muted-foreground)" }}>06</span>
					{"\u00a0\u00a0"}
					{t("own.label")}
				</p>
				<div>
					<h2
						className="h2"
						style={{
							margin: "0",
							fontWeight: "500",
							letterSpacing: "-0.025em",
							textWrap: "balance",
						}}
					>
						{t("own.a")}
						<br />
						<span style={{ color: "var(--muted-foreground)" }}>{t("own.b")}</span>
					</h2>
					<p className="body" style={{ margin: "36px 0 0" }}>
						{t("own.p")}
					</p>
				</div>
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
					padding: "clamp(32px, 5vw, 64px) 16px",
				}}
			>
				<Waves seed={6} />
				<pre
					className="code"
					style={{
						position: "relative",
						margin: "0",
						width: "min(680px, 100%)",
						boxSizing: "border-box",
						padding: "22px 24px",
						borderRadius: "12px",
						background: "#0b0b0c",
						color: "#e4e4e7",
						fontFamily: "var(--font-mono)",
						fontSize: "14px",
						lineHeight: "1.75",
						whiteSpace: "pre-wrap",
						overflowX: "auto",
					}}
				>
					<span style={{ color: "#8e8e99" }}>{"$"}</span>
					{" scripts/new-content.sh page countdown\n"}
					<span style={{ color: "#8e8e99" }}>{"›"}</span>
					{" /add-content a page that counts down to a date   "}
					<span style={{ color: "#6e6e7a" }}>{"# in Claude Code"}</span>
					{"\n"}
					<span style={{ color: "#8e8e99" }}>{"$"}</span>
					{" scripts/check-content-paths.sh --id countdown"}
				</pre>
			</div>
			<div className="cards">
				{ownSteps.map((st) => (
					<div
						style={{
							borderTop: "1px solid var(--border)",
							paddingTop: "18px",
							display: "flex",
							flexDirection: "column",
							gap: "6px",
						}}
						key={st.t}
					>
						<span style={{ fontSize: "18px", fontWeight: "500" }}>{st.t}</span>
						<span
							style={{ fontSize: "15px", lineHeight: "1.55", color: "var(--muted-foreground)" }}
						>
							{st.d}
						</span>
					</div>
				))}
			</div>
		</section>
	);
}
