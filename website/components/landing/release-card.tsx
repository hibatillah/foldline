import Image from "next/image";
import { useTranslations } from "next-intl";

export function ReleaseCard() {
	const t = useTranslations();

	return (
		<div
			style={{
				width: "min(480px, calc(100% - 32px))",
				background: "var(--card)",
				borderRadius: "12px",
				boxShadow:
					"0 0 0 1px color-mix(in oklch, var(--foreground) 9%, transparent), 0 18px 40px -22px rgba(15,23,42,0.35)",
				textAlign: "left",
				fontSize: "14px",
			}}
		>
			<div
				style={{
					display: "flex",
					alignItems: "center",
					gap: "10px",
					padding: "16px 18px",
					borderBottom: "1px solid var(--border)",
				}}
			>
				<Image src="/logo.svg" alt="" width={22} height={22} style={{ borderRadius: "6px" }} />
				<span style={{ fontWeight: "600" }}>Foldline v0.6.0</span>
				<span
					style={{
						marginLeft: "auto",
						fontSize: "12px",
						fontWeight: "500",
						color: "var(--primary)",
					}}
				>
					{t("install.latest")}
				</span>
			</div>
			<div style={{ padding: "6px 18px 8px" }}>
				<div
					style={{
						display: "flex",
						alignItems: "center",
						gap: "12px",
						padding: "12px 0",
						borderBottom: "1px solid var(--border)",
					}}
				>
					<svg
						aria-hidden="true"
						width="16"
						height="16"
						viewBox="0 0 24 24"
						fill="none"
						stroke="var(--muted-foreground)"
						strokeWidth="2"
						strokeLinecap="round"
						strokeLinejoin="round"
					>
						<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
					</svg>
					<span style={{ display: "flex", flexDirection: "column", minWidth: "0" }}>
						<span
							style={{
								fontFamily: "var(--font-mono)",
								fontSize: "13px",
								whiteSpace: "nowrap",
								overflow: "hidden",
								textOverflow: "ellipsis",
							}}
						>
							foldline-v0.6.0-windows-x64-setup.exe
						</span>
						<span style={{ fontSize: "12px", color: "var(--muted-foreground)" }}>
							{t("install.installer")}
						</span>
					</span>
					<span
						style={{
							marginLeft: "auto",
							fontSize: "12px",
							color: "var(--muted-foreground)",
							whiteSpace: "nowrap",
							fontVariantNumeric: "tabular-nums",
						}}
					>
						5.75 MB
					</span>
				</div>
				<div
					style={{
						display: "flex",
						alignItems: "center",
						gap: "12px",
						padding: "12px 0",
						borderBottom: "1px solid var(--border)",
					}}
				>
					<svg
						aria-hidden="true"
						width="16"
						height="16"
						viewBox="0 0 24 24"
						fill="none"
						stroke="var(--muted-foreground)"
						strokeWidth="2"
						strokeLinecap="round"
						strokeLinejoin="round"
					>
						<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
					</svg>
					<span style={{ display: "flex", flexDirection: "column", minWidth: "0" }}>
						<span
							style={{
								fontFamily: "var(--font-mono)",
								fontSize: "13px",
								whiteSpace: "nowrap",
								overflow: "hidden",
								textOverflow: "ellipsis",
							}}
						>
							foldline.exe
						</span>
						<span style={{ fontSize: "12px", color: "var(--muted-foreground)" }}>
							{t("install.portable")}
						</span>
					</span>
					<span
						style={{
							marginLeft: "auto",
							fontSize: "12px",
							color: "var(--muted-foreground)",
							whiteSpace: "nowrap",
							fontVariantNumeric: "tabular-nums",
						}}
					/>
				</div>
			</div>
			<div
				style={{ padding: "10px 18px 16px", fontSize: "13px", color: "var(--muted-foreground)" }}
			>
				{t("install.req")}
			</div>
		</div>
	);
}
