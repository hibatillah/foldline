import { useTranslations } from "next-intl";
import type en from "@/messages/en.json";
import { CtaLink } from "./cta-link";
import { HeroFrame } from "./hero-frame";
import { WindowsIcon } from "./windows-icon";

type Messages = typeof en;

export function Hero() {
	const t = useTranslations();
	const heroList = t.raw("heroList") as Messages["heroList"];

	return (
		<section className="wrap" style={{ paddingTop: "clamp(100px, 5vw, 64px)" }}>
			<h1 className="h1" style={{ margin: "0", fontWeight: "500", letterSpacing: "-0.03em" }}>
				<span style={{ color: "var(--muted-foreground)" }}>{t("hero.b")}</span> {t("hero.a")}
			</h1>
			<div
				style={{
					display: "flex",
					flexWrap: "wrap",
					alignItems: "center",
					gap: "20px",
					marginTop: "44px",
				}}
			>
				<CtaLink href="/download" size="lg" variant="primary">
					<WindowsIcon />
					{t("hero.cta")}
				</CtaLink>
				<CtaLink
					href="https://github.com/hibatillah/foldline/blob/master/CONTRIBUTING.md#add-your-own-content"
					size="lg"
					variant="outline"
				>
					{t("hero.own")}
				</CtaLink>
				<span style={{ fontSize: "14px", color: "var(--muted-foreground)" }}>{t("hero.meta")}</span>
			</div>
			<div className="hero" style={{ marginTop: "64px" }}>
				<div className="herof">
					<HeroFrame />
				</div>
				<div
					style={{
						display: "flex",
						flexDirection: "column",
						justifyContent: "space-between",
						gap: "32px",
						padding: "4px 0",
					}}
				>
					<h2
						className="h3"
						style={{
							margin: "0",
							fontWeight: "500",
							letterSpacing: "-0.025em",
							textWrap: "balance",
						}}
					>
						{t("hero.side1")}
						<br />
						<span style={{ color: "var(--muted-foreground)" }}>{t("hero.side2")}</span>
					</h2>
					<div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
						<p
							style={{
								margin: "0",
								display: "flex",
								flexWrap: "wrap",
								rowGap: "6px",
								fontSize: "16px",
								fontWeight: "500",
								lineHeight: "1.5",
							}}
						>
							{heroList.map((item, index) => (
								<span key={item}>
									{item}
									{index < heroList.length - 1 && (
										<span style={{ color: "var(--primary)", padding: "0 10px" }}>·</span>
									)}
								</span>
							))}
						</p>
					</div>
				</div>
			</div>
		</section>
	);
}
