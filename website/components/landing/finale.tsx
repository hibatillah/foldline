import { useTranslations } from "next-intl";
import { CtaLink } from "./cta-link";
import { WindowsIcon } from "./windows-icon";

export function Finale() {
	const t = useTranslations();

	return (
		<section className="wrap" style={{ paddingTop: "clamp(120px, 13vw, 180px)" }}>
			<div
				style={{
					position: "relative",
					overflow: "hidden",
					borderRadius: "20px",
					background: "linear-gradient(160deg, #4aa3ff 0%, #2f7cf2 45%, #1c5fe0 100%)",
					padding: "clamp(64px, 7vw, 96px) 24px",
					textAlign: "center",
					display: "flex",
					flexDirection: "column",
					alignItems: "center",
					gap: "28px",
				}}
			>
				<div
					aria-hidden="true"
					style={{
						position: "absolute",
						inset: "0",
						background:
							"linear-gradient(135deg, rgba(255,255,255,0.28) 0%, rgba(255,255,255,0.06) 45%, rgba(255,255,255,0.1) 100%)",
						pointerEvents: "none",
					}}
				/>
				<div
					aria-hidden="true"
					style={{
						position: "absolute",
						left: "50%",
						top: "0",
						transform: "translateX(-50%)",
						width: "150px",
						height: "33px",
						background: "#000",
						borderRadius: "0 0 12px 12px",
					}}
				>
					<span
						aria-hidden="true"
						style={{
							position: "absolute",
							left: "-6px",
							top: "0",
							width: "6px",
							height: "6px",
							background: "radial-gradient(circle 6px at 0 100%, transparent 98%, #000 100%)",
						}}
					/>
					<span
						aria-hidden="true"
						style={{
							position: "absolute",
							right: "-6px",
							top: "0",
							width: "6px",
							height: "6px",
							background: "radial-gradient(circle 6px at 100% 100%, transparent 98%, #000 100%)",
						}}
					/>
				</div>
				<h2
					className="h2"
					style={{
						position: "relative",
						margin: "0",
						fontWeight: "500",
						letterSpacing: "-0.025em",
						color: "#fff",
					}}
				>
					{t("final.a")}
					<br />
					<span style={{ color: "rgba(255,255,255,0.72)" }}>{t("final.b")}</span>
				</h2>
				<div
					style={{
						position: "relative",
						display: "flex",
						flexWrap: "wrap",
						justifyContent: "center",
						gap: "12px",
					}}
				>
					<CtaLink href="/download" size="lg" variant="light">
						<WindowsIcon />
						{t("hero.cta")}
					</CtaLink>
					<CtaLink
						href="https://github.com/hibatillah/foldline/releases"
						size="lg"
						variant="lightOutline"
					>
						{t("final.notes")}
					</CtaLink>
				</div>
			</div>
		</section>
	);
}
