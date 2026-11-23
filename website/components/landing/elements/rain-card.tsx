import { useTranslations } from "next-intl";

export function RainCard() {
	const t = useTranslations();

	return (
		<div
			style={{
				display: "flex",
				flexDirection: "column",
				alignItems: "center",
				gap: "14px",
				color: "#18181b",
			}}
		>
			<svg
				aria-hidden="true"
				width="44"
				height="44"
				viewBox="0 0 24 24"
				fill="none"
				stroke="#1c7ed6"
				strokeWidth="2"
				strokeLinecap="round"
				strokeLinejoin="round"
			>
				<path d="M4 14.9A7 7 0 1 1 15.7 8h1.8a4.5 4.5 0 0 1 2.5 8.2" />
				<path d="M16 14v6M8 14v6M12 16v6" />
			</svg>
			<div style={{ textAlign: "center" }}>
				<div style={{ fontSize: "16px", fontWeight: "500" }}>{t("word.rainIn")}</div>
				<div style={{ fontSize: "13px", color: "#71717a", marginTop: "3px" }}>
					{t("word.rainThen")}
				</div>
			</div>
		</div>
	);
}
