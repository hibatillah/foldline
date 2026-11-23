export function StatsGrid() {
	return (
		<div style={{ display: "grid", gridTemplateColumns: "repeat(2, 132px)", gap: "8px" }}>
			<div
				style={{
					position: "relative",
					width: "132px",
					height: "86px",
					borderRadius: "12px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					color: "#18181b",
				}}
			>
				<span
					style={{
						position: "absolute",
						left: "12px",
						top: "8px",
						fontSize: "11px",
						color: "#71717a",
					}}
				>
					CPU
				</span>
				<svg
					aria-hidden="true"
					width="132"
					height="86"
					viewBox="0 0 132 86"
					style={{ position: "absolute", inset: "0" }}
				>
					<line
						x1="12"
						x2="118"
						y1="60"
						y2="60"
						stroke="#a1a1aa"
						strokeWidth="1.4"
						strokeDasharray="1.4 4"
						strokeLinecap="round"
					/>
					<rect x="110.2" y="56" width="2.6" height="4" rx="1" fill="#a5d8ff" />
					<rect x="114.2" y="53" width="2.6" height="7" rx="1" fill="#1c7ed6" />
				</svg>
				<span
					style={{
						position: "absolute",
						left: "12px",
						top: "22px",
						fontSize: "20px",
						fontWeight: "500",
					}}
				>
					12%
				</span>
			</div>
			<div
				style={{
					position: "relative",
					width: "132px",
					height: "86px",
					borderRadius: "12px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					color: "#18181b",
				}}
			>
				<span
					style={{
						position: "absolute",
						left: "12px",
						top: "8px",
						fontSize: "11px",
						color: "#71717a",
					}}
				>
					Memory
				</span>
				<svg
					aria-hidden="true"
					width="132"
					height="86"
					viewBox="0 0 132 86"
					style={{ position: "absolute", inset: "0" }}
				>
					<line
						x1="12"
						x2="118"
						y1="60"
						y2="60"
						stroke="#a1a1aa"
						strokeWidth="1.4"
						strokeDasharray="1.4 4"
						strokeLinecap="round"
					/>
					<rect x="110.2" y="52" width="2.6" height="8" rx="1" fill="#eebefa" />
					<rect x="114.2" y="52" width="2.6" height="8" rx="1" fill="#ae3ec9" />
				</svg>
				<span
					style={{
						position: "absolute",
						left: "12px",
						top: "22px",
						fontSize: "20px",
						fontWeight: "500",
					}}
				>
					61%
				</span>
			</div>
			<div
				style={{
					position: "relative",
					width: "132px",
					height: "86px",
					borderRadius: "12px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					color: "#18181b",
				}}
			>
				<span
					style={{
						position: "absolute",
						left: "12px",
						top: "8px",
						fontSize: "11px",
						color: "#71717a",
					}}
				>
					GPU
				</span>
				<svg
					aria-hidden="true"
					width="132"
					height="86"
					viewBox="0 0 132 86"
					style={{ position: "absolute", inset: "0" }}
				>
					<line
						x1="12"
						x2="118"
						y1="60"
						y2="60"
						stroke="#a1a1aa"
						strokeWidth="1.4"
						strokeDasharray="1.4 4"
						strokeLinecap="round"
					/>
					<rect x="110.2" y="57" width="2.6" height="3" rx="1" fill="#ffd8a8" />
					<rect x="114.2" y="55" width="2.6" height="5" rx="1" fill="#f08c00" />
				</svg>
				<span
					style={{
						position: "absolute",
						left: "12px",
						top: "22px",
						fontSize: "20px",
						fontWeight: "500",
					}}
				>
					8%
				</span>
			</div>
			<div
				style={{
					position: "relative",
					width: "132px",
					height: "86px",
					borderRadius: "12px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					color: "#18181b",
				}}
			>
				<span
					style={{
						position: "absolute",
						left: "12px",
						top: "8px",
						fontSize: "11px",
						color: "#71717a",
					}}
				>
					Network
				</span>
				<span
					style={{
						position: "absolute",
						left: "12px",
						top: "24px",
						fontSize: "14px",
						fontWeight: "500",
					}}
				>
					↓ 2.4 MB/s
				</span>
				<span
					style={{
						position: "absolute",
						left: "12px",
						top: "46px",
						fontSize: "13px",
						color: "#71717a",
					}}
				>
					↑ 180 KB/s
				</span>
				<span
					style={{
						position: "absolute",
						left: "12px",
						top: "66px",
						fontSize: "11px",
						color: "#71717a",
					}}
				>
					Wi-Fi
				</span>
			</div>
		</div>
	);
}
