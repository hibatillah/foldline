import Image from "next/image";
import { useTranslations } from "next-intl";
import { Link } from "@/i18n/navigation";
import type en from "@/messages/en.json";
import { ExternalLink } from "./external-link";
import { FeedbackDialog } from "./feedback-dialog";
import { ExternalArrow } from "./external-arrow";

type Messages = typeof en;

export function SiteFooter() {
	const t = useTranslations();
	const foot = t.raw("foot") as Messages["foot"];

	return (
		<footer style={{ position: "relative", overflow: "hidden", marginTop: "96px" }}>
			<div
				aria-hidden="true"
				className="wordmark"
				style={{
					position: "absolute",
					left: "50%",
					bottom: "-0.2em",
					transform: "translateX(-50%)",
				}}
			>
				Foldline
			</div>
			<div
				className="wrap"
				style={{
					position: "relative",
					paddingTop: "24px",
					paddingBottom: "clamp(56px, 6vw, 88px)",
				}}
			>
				<div
					className="foot"
					style={{
						display: "grid",
						gridTemplateColumns: "minmax(0, 2fr) repeat(3, minmax(0, 1fr))",
						gap: "32px",
						paddingBottom: "48px",
					}}
				>
					<div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
						<span
							style={{
								display: "flex",
								alignItems: "center",
								gap: "10px",
								fontWeight: "600",
								fontSize: "16px",
							}}
						>
							<Image
								src="/logo.svg"
								alt=""
								width={22}
								height={22}
								style={{ borderRadius: "6px" }}
							/>
							Foldline
						</span>
						<span
							style={{
								fontSize: "14px",
								lineHeight: "1.55",
								color: "var(--muted-foreground)",
								maxWidth: "260px",
							}}
						>
							{t("footTag")}
						</span>
					</div>
					{foot.map((fc) => (
						<div
							style={{ display: "flex", flexDirection: "column", gap: "10px", fontSize: "14px" }}
							key={fc.h}
						>
							<span style={{ fontWeight: "500" }}>{fc.h}</span>
							{fc.links.map((link) => {
								const style = {
									textDecoration: "none",
									color: "var(--muted-foreground)",
									display: "inline-flex",
									alignItems: "center",
									gap: "6px",
								};
								if (link.href === "dialog:feedback") {
									return <FeedbackDialog label={link.label} key={link.label} />;
								}
								// Anchors and pages stay on the site, so they work from the policy pages too.
								if (!link.href.startsWith("http")) {
									return (
										<Link
											href={link.href.startsWith("#") ? `/${link.href}` : link.href}
											style={style}
											key={link.label}
										>
											{link.label}
										</Link>
									);
								}
								return (
									<ExternalLink href={link.href} style={style} key={link.label}>
										{link.label}
										<ExternalArrow />
									</ExternalLink>
								);
							})}
						</div>
					))}
				</div>
				<div
					style={{
						display: "flex",
						flexWrap: "wrap",
						justifyContent: "space-between",
						gap: "12px",
						paddingTop: "20px",
						fontSize: "13px",
						color: "var(--muted-foreground)",
					}}
				>
					<span>{"© 2026 hibatillah · GPL-3.0-or-later"}</span>
					<span>{t("legal")}</span>
				</div>
			</div>
		</footer>
	);
}
