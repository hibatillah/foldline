export function WeatherCells() {
	return (
		<div style={{ display: "grid", gridTemplateColumns: "repeat(3, 62px)", gap: "6px" }}>
			<div
				style={{
					height: "86px",
					borderRadius: "10px",
					background: "#fff4e0",
					boxShadow: "0 0 0 1px #ffe0a8",
					display: "flex",
					flexDirection: "column",
					alignItems: "center",
					justifyContent: "center",
					gap: "4px",
					color: "#18181b",
				}}
			>
				<span style={{ fontSize: "11px", color: "#b5651d" }}>Now</span>
				<svg
					aria-hidden="true"
					width="20"
					height="20"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#f08c00"
					strokeWidth="2"
					strokeLinecap="round"
					strokeLinejoin="round"
				>
					<path d="M12 2v2M4.9 4.9l1.4 1.4M20 12h2M19.1 4.9l-1.4 1.4M15.9 13.1A4 4 0 0 0 8.4 9.6" />
					<path d="M13 22H7a5 5 0 1 1 4.9-6H13a3 3 0 0 1 0 6z" />
				</svg>
				<span style={{ fontSize: "13px" }}>31°</span>
				<span style={{ fontSize: "11px", color: "#1c7ed6", height: "13px" }} />
			</div>
			<div
				style={{
					height: "86px",
					borderRadius: "10px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					display: "flex",
					flexDirection: "column",
					alignItems: "center",
					justifyContent: "center",
					gap: "4px",
					color: "#18181b",
				}}
			>
				<span style={{ fontSize: "11px", color: "#71717a" }}>16:00</span>
				<svg
					aria-hidden="true"
					width="20"
					height="20"
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
				<span style={{ fontSize: "13px" }}>29°</span>
				<span style={{ fontSize: "11px", color: "#1c7ed6", height: "13px" }}>1.2</span>
			</div>
			<div
				style={{
					height: "86px",
					borderRadius: "10px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					display: "flex",
					flexDirection: "column",
					alignItems: "center",
					justifyContent: "center",
					gap: "4px",
					color: "#18181b",
				}}
			>
				<span style={{ fontSize: "11px", color: "#71717a" }}>17:00</span>
				<svg
					aria-hidden="true"
					width="20"
					height="20"
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
				<span style={{ fontSize: "13px" }}>27°</span>
				<span style={{ fontSize: "11px", color: "#1c7ed6", height: "13px" }}>3.4</span>
			</div>
			<div
				style={{
					height: "86px",
					borderRadius: "10px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					display: "flex",
					flexDirection: "column",
					alignItems: "center",
					justifyContent: "center",
					gap: "4px",
					color: "#18181b",
				}}
			>
				<span style={{ fontSize: "11px", color: "#71717a" }}>18:00</span>
				<svg
					aria-hidden="true"
					width="20"
					height="20"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#868e96"
					strokeWidth="2"
					strokeLinecap="round"
					strokeLinejoin="round"
				>
					<path d="M17.5 19H9a7 7 0 1 1 6.7-9h1.8a4.5 4.5 0 1 1 0 9z" />
				</svg>
				<span style={{ fontSize: "13px" }}>27°</span>
				<span style={{ fontSize: "11px", color: "#1c7ed6", height: "13px" }}>0.4</span>
			</div>
			<div
				style={{
					height: "86px",
					borderRadius: "10px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					display: "flex",
					flexDirection: "column",
					alignItems: "center",
					justifyContent: "center",
					gap: "4px",
					color: "#18181b",
				}}
			>
				<span style={{ fontSize: "11px", color: "#71717a" }}>19:00</span>
				<svg
					aria-hidden="true"
					width="20"
					height="20"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#868e96"
					strokeWidth="2"
					strokeLinecap="round"
					strokeLinejoin="round"
				>
					<path d="M17.5 19H9a7 7 0 1 1 6.7-9h1.8a4.5 4.5 0 1 1 0 9z" />
				</svg>
				<span style={{ fontSize: "13px" }}>26°</span>
				<span style={{ fontSize: "11px", color: "#1c7ed6", height: "13px" }} />
			</div>
			<div
				style={{
					height: "86px",
					borderRadius: "10px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					display: "flex",
					flexDirection: "column",
					alignItems: "center",
					justifyContent: "center",
					gap: "4px",
					color: "#18181b",
				}}
			>
				<span style={{ fontSize: "11px", color: "#71717a" }}>20:00</span>
				<svg
					aria-hidden="true"
					width="20"
					height="20"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#868e96"
					strokeWidth="2"
					strokeLinecap="round"
					strokeLinejoin="round"
				>
					<path d="M17.5 19H9a7 7 0 1 1 6.7-9h1.8a4.5 4.5 0 1 1 0 9z" />
				</svg>
				<span style={{ fontSize: "13px" }}>26°</span>
				<span style={{ fontSize: "11px", color: "#1c7ed6", height: "13px" }} />
			</div>
		</div>
	);
}
