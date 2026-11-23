import { OpenNotch } from "./open-notch";

export function WeatherNotch() {
	return (
		<OpenNotch width={526} activePage={6}>
			<div style={{ height: "86px", display: "flex", gap: "16px" }}>
				<div style={{ width: "146px", flex: "none", display: "flex", flexDirection: "column" }}>
					<div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
						<span style={{ fontSize: "10px", color: "#9a9aa6" }}>Pekanbaru</span>
						<span
							style={{
								height: "22px",
								padding: "0 9px",
								borderRadius: "11px",
								background: "#1c1c22",
								border: "1px solid #2c2c34",
								boxSizing: "border-box",
								fontSize: "11px",
								color: "#e6e6eb",
								display: "flex",
								alignItems: "center",
							}}
						>
							Today
						</span>
					</div>
					<span style={{ fontSize: "28px", fontWeight: "500", lineHeight: "32px" }}>31°</span>
					<span style={{ fontSize: "11px", color: "#9a9aa6" }}>Partly cloudy · feels 35°</span>
					<span style={{ fontSize: "12px", color: "#74c0fc", marginTop: "2px" }}>
						Rain from 16:00
					</span>
				</div>
				<div
					style={{
						flex: "1",
						display: "grid",
						gridTemplateColumns: "repeat(6, minmax(0, 1fr))",
						gap: "4px",
					}}
				>
					<div
						style={{
							borderRadius: "10px",
							background: "#2b2312",
							display: "flex",
							flexDirection: "column",
							alignItems: "center",
							justifyContent: "center",
							gap: "4px",
						}}
					>
						<span style={{ fontSize: "10px", color: "#ffc078" }}>Now</span>
						<svg
							aria-hidden="true"
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#ffc078"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d="M12 2v2M4.9 4.9l1.4 1.4M20 12h2M19.1 4.9l-1.4 1.4M15.9 13.1A4 4 0 0 0 8.4 9.6" />
							<path d="M13 22H7a5 5 0 1 1 4.9-6H13a3 3 0 0 1 0 6z" />
						</svg>
						<span style={{ fontSize: "13px" }}>31°</span>
						<span style={{ fontSize: "10px", color: "#74c0fc", height: "12px" }} />
					</div>
					<div
						style={{
							borderRadius: "10px",
							display: "flex",
							flexDirection: "column",
							alignItems: "center",
							justifyContent: "center",
							gap: "4px",
						}}
					>
						<span style={{ fontSize: "10px", color: "#9a9aa6" }}>16:00</span>
						<svg
							aria-hidden="true"
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#74c0fc"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d="M4 14.9A7 7 0 1 1 15.7 8h1.8a4.5 4.5 0 0 1 2.5 8.2" />
							<path d="M16 14v6M8 14v6M12 16v6" />
						</svg>
						<span style={{ fontSize: "13px" }}>29°</span>
						<span style={{ fontSize: "10px", color: "#74c0fc", height: "12px" }}>1.2</span>
					</div>
					<div
						style={{
							borderRadius: "10px",
							display: "flex",
							flexDirection: "column",
							alignItems: "center",
							justifyContent: "center",
							gap: "4px",
						}}
					>
						<span style={{ fontSize: "10px", color: "#9a9aa6" }}>17:00</span>
						<svg
							aria-hidden="true"
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#74c0fc"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d="M4 14.9A7 7 0 1 1 15.7 8h1.8a4.5 4.5 0 0 1 2.5 8.2" />
							<path d="M16 14v6M8 14v6M12 16v6" />
						</svg>
						<span style={{ fontSize: "13px" }}>27°</span>
						<span style={{ fontSize: "10px", color: "#74c0fc", height: "12px" }}>3.4</span>
					</div>
					<div
						style={{
							borderRadius: "10px",
							display: "flex",
							flexDirection: "column",
							alignItems: "center",
							justifyContent: "center",
							gap: "4px",
						}}
					>
						<span style={{ fontSize: "10px", color: "#9a9aa6" }}>18:00</span>
						<svg
							aria-hidden="true"
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#c4c4cc"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d="M17.5 19H9a7 7 0 1 1 6.7-9h1.8a4.5 4.5 0 1 1 0 9z" />
						</svg>
						<span style={{ fontSize: "13px" }}>27°</span>
						<span style={{ fontSize: "10px", color: "#74c0fc", height: "12px" }}>0.4</span>
					</div>
					<div
						style={{
							borderRadius: "10px",
							display: "flex",
							flexDirection: "column",
							alignItems: "center",
							justifyContent: "center",
							gap: "4px",
						}}
					>
						<span style={{ fontSize: "10px", color: "#9a9aa6" }}>19:00</span>
						<svg
							aria-hidden="true"
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#c4c4cc"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d="M17.5 19H9a7 7 0 1 1 6.7-9h1.8a4.5 4.5 0 1 1 0 9z" />
						</svg>
						<span style={{ fontSize: "13px" }}>26°</span>
						<span style={{ fontSize: "10px", color: "#74c0fc", height: "12px" }} />
					</div>
					<div
						style={{
							borderRadius: "10px",
							display: "flex",
							flexDirection: "column",
							alignItems: "center",
							justifyContent: "center",
							gap: "4px",
						}}
					>
						<span style={{ fontSize: "10px", color: "#9a9aa6" }}>20:00</span>
						<svg
							aria-hidden="true"
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#c4c4cc"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d="M17.5 19H9a7 7 0 1 1 6.7-9h1.8a4.5 4.5 0 1 1 0 9z" />
						</svg>
						<span style={{ fontSize: "13px" }}>26°</span>
						<span style={{ fontSize: "10px", color: "#74c0fc", height: "12px" }} />
					</div>
				</div>
			</div>
		</OpenNotch>
	);
}
