import Image from "next/image";
import type { ReactNode } from "react";

import { Link } from "@/i18n/navigation";

import { LanguageSwitch } from "./language-switch";
import { SiteFooter } from "./site-footer";

/** The frame of a page that is not the landing page: a slim header, the content and the site footer. */
export function PageShell({ children }: { children: ReactNode }) {
	return (
		<div
			className="landing"
			style={{
				fontFamily: "var(--font-sans)",
				color: "var(--foreground)",
				background: "var(--background)",
				WebkitFontSmoothing: "antialiased",
			}}
		>
			<header className="wrap" style={{ paddingTop: "24px" }}>
				<div
					style={{
						display: "flex",
						alignItems: "center",
						justifyContent: "space-between",
						gap: "16px",
					}}
				>
					<Link
						href="/"
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
					</Link>
					<LanguageSwitch />
				</div>
			</header>
			<main className="wrap" style={{ paddingTop: "56px" }}>
				{children}
			</main>
			<SiteFooter />
		</div>
	);
}
