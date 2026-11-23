export function FocusTimer() {
	return (
		<div style={{ width: "100%", maxWidth: "300px", color: "#18181b" }}>
			<div style={{ display: "flex", alignItems: "flex-end", gap: "12px" }}>
				<span
					style={{
						fontSize: "44px",
						fontWeight: "500",
						lineHeight: "46px",
						letterSpacing: "-0.01em",
						fontVariantNumeric: "tabular-nums",
					}}
				>
					25:00
				</span>
				<span style={{ display: "flex", flexDirection: "column", paddingBottom: "6px" }}>
					<span style={{ fontSize: "14px", fontWeight: "500" }}>Focus</span>
					<span style={{ fontSize: "12px", color: "#71717a" }}>Session 1 of 4</span>
				</span>
			</div>
			<div
				style={{
					marginTop: "16px",
					display: "grid",
					gridTemplateColumns: "repeat(2, minmax(0, 1fr))",
					gap: "9px",
				}}
			>
				<span
					style={{
						height: "34px",
						borderRadius: "17px",
						background: "#ff9e0a",
						color: "#1a1205",
						fontSize: "13px",
						fontWeight: "500",
						display: "flex",
						alignItems: "center",
						justifyContent: "center",
					}}
				>
					Start
				</span>
				<span
					style={{
						height: "34px",
						borderRadius: "17px",
						background: "#ffffff",
						boxShadow: "0 0 0 1px #e4e4e7",
						fontSize: "13px",
						fontWeight: "500",
						display: "flex",
						alignItems: "center",
						justifyContent: "center",
					}}
				>
					Short break
				</span>
			</div>
		</div>
	);
}
