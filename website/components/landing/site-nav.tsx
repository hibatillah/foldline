import Image from "next/image";
import { useTranslations } from "next-intl";
import { ExternalLink } from "./external-link";
import { ExternalArrow } from "./external-arrow";
import { LanguageSwitch } from "./language-switch";
import { CtaLink } from "./cta-link";

export function SiteNav() {
	const t = useTranslations();

	return (
		<header
			style={{
				position: "sticky",
				top: "12px",
				zIndex: "20",
				padding: "12px clamp(16px, 3vw, 40px) 0",
			}}
		>
			<nav
				aria-label={t("navAria")}
				style={{
					maxWidth: "1240px",
					margin: "0 auto",
					boxSizing: "border-box",
					padding: "0 12px 0 20px",
					borderRadius: "16px",
					background: "rgba(255,255,255,0.72)",
					backdropFilter: "saturate(180%) blur(16px)",
					WebkitBackdropFilter: "saturate(180%) blur(16px)",
					boxShadow:
						"0 0 0 1px color-mix(in oklch, var(--foreground) 8%, transparent), 0 12px 32px -18px rgba(15,23,42,0.28)",
					height: "60px",
					display: "grid",
					gridTemplateColumns: "1fr auto 1fr",
					alignItems: "center",
					gap: "24px",
				}}
			>
				<a
					href="#top"
					style={{
						display: "flex",
						alignItems: "center",
						gap: "10px",
						textDecoration: "none",
						fontWeight: "600",
						fontSize: "18px",
						letterSpacing: "-0.01em",
					}}
				>
					<Image
						src="/logo.svg"
						alt=""
						width={26}
						height={26}
						style={{ display: "block", borderRadius: "7px" }}
					/>
					Foldline
				</a>
				<div className="navlinks" style={{ gap: "30px", fontSize: "15px" }}>
					<a href="#features" style={{ textDecoration: "none" }}>
						{t("nav.features")}
					</a>
					<a href="#install" style={{ textDecoration: "none", color: "var(--muted-foreground)" }}>
						{t("nav.install")}
					</a>
					<a href="#faq" style={{ textDecoration: "none", color: "var(--muted-foreground)" }}>
						{t("nav.faq")}
					</a>
					<ExternalLink
						href="https://fold.hibatillah.com/releases"
						style={{
							textDecoration: "none",
							color: "var(--muted-foreground)",
							display: "inline-flex",
							alignItems: "center",
							gap: "6px",
						}}
					>
						{t("nav.releases")}
						<ExternalArrow />
					</ExternalLink>
				</div>
				<div
					style={{ display: "flex", alignItems: "center", justifyContent: "flex-end", gap: "16px" }}
				>
					<LanguageSwitch />
					<ExternalLink
						className="gh"
						href="https://github.com/hibatillah/foldline"
						style={{
							textDecoration: "none",
							fontSize: "15px",
							fontWeight: "500",
							display: "inline-flex",
							alignItems: "center",
							gap: "6px",
						}}
					>
						GitHub
						<ExternalArrow />
					</ExternalLink>
					<CtaLink href="/download" size="sm" variant="primary">
						{t("nav.cta")}
					</CtaLink>
				</div>
			</nav>
		</header>
	);
}
