import { useTranslations } from "next-intl";

export function StatusTiles() {
	const t = useTranslations();

	return (
		<div
			style={{
				width: "100%",
				maxWidth: "420px",
				display: "grid",
				gridTemplateColumns: "repeat(2, minmax(0, 1fr))",
				gap: "10px",
			}}
		>
			<div
				style={{
					height: "56px",
					borderRadius: "12px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					display: "flex",
					alignItems: "center",
					gap: "12px",
					padding: "0 16px",
					fontSize: "14px",
					color: "#18181b",
				}}
			>
				<svg
					aria-hidden="true"
					width="18"
					height="18"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#18181b"
					strokeWidth="2"
					strokeLinejoin="round"
				>
					<path d="M12 4 4 12h4v5h8v-5h4z" />
					<path d="M8 20h8" />
				</svg>
				<span>Caps Lock</span>
			</div>
			<div
				style={{
					height: "56px",
					borderRadius: "12px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					display: "flex",
					alignItems: "center",
					gap: "12px",
					padding: "0 16px",
					fontSize: "14px",
					color: "#18181b",
				}}
			>
				<svg
					aria-hidden="true"
					width="18"
					height="18"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#18181b"
					strokeWidth="2"
				>
					<path d="M11 5 6 9H3v6h3l5 4z" />
					<path d="m16 9 5 6M21 9l-5 6" />
				</svg>
				<span>{t("word.muted")}</span>
			</div>
			<div
				style={{
					height: "56px",
					borderRadius: "12px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					display: "flex",
					alignItems: "center",
					gap: "12px",
					padding: "0 16px",
					fontSize: "14px",
					color: "#18181b",
				}}
			>
				<span
					style={{
						width: "10px",
						height: "10px",
						borderRadius: "50%",
						background: "#2f9e44",
						margin: "0 4px",
					}}
				/>
				<span>{t("word.camera")}</span>
			</div>
			<div
				style={{
					height: "56px",
					borderRadius: "12px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					display: "flex",
					alignItems: "center",
					gap: "12px",
					padding: "0 16px",
					fontSize: "14px",
					color: "#18181b",
				}}
			>
				<span
					style={{
						width: "10px",
						height: "10px",
						borderRadius: "50%",
						background: "#f08c00",
						margin: "0 4px",
					}}
				/>
				<span>{t("word.microphone")}</span>
			</div>
		</div>
	);
}
