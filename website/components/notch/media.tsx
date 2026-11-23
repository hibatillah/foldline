import { OpenNotch, MediaStatus } from "./open-notch";

export function MediaNotch() {
	return (
		<OpenNotch
			width={614}
			activePage={0}
			background="linear-gradient(180deg, #000 0%, #04121a 35%, #174b57 100%)"
			status={<MediaStatus />}
		>
			<div style={{ height: "86px", display: "flex", gap: "20px", alignItems: "center" }}>
				<div
					style={{
						position: "relative",
						flex: "none",
						width: "86px",
						height: "86px",
						borderRadius: "12px",
						background:
							"radial-gradient(ellipse 30% 55% at 50% 0%, #cfe4e8 0%, rgba(80,120,130,0.55) 45%, rgba(10,24,28,0) 100%), linear-gradient(180deg, #0b1a1d, #14292d)",
					}}
				>
					<span
						style={{
							position: "absolute",
							left: "38%",
							bottom: "20%",
							width: "22%",
							height: "34%",
							borderRadius: "40% 40% 30% 30%",
							background: "#d8c9c0",
							opacity: "0.8",
						}}
					/>
					<span
						style={{
							position: "absolute",
							right: "-1px",
							bottom: "-1px",
							width: "19px",
							height: "19px",
							borderRadius: "5px",
							background: "#0c0c0c",
							boxShadow: "0 0 0 3px #000",
							display: "flex",
							alignItems: "center",
							justifyContent: "center",
						}}
					>
						<span
							style={{ width: "11px", height: "11px", borderRadius: "50%", background: "#e6e6eb" }}
						/>
					</span>
				</div>
				<div
					style={{
						flex: "none",
						width: "170px",
						display: "flex",
						flexDirection: "column",
						gap: "2px",
					}}
				>
					<div style={{ fontSize: "14px", fontWeight: "500" }}>Late Bloom</div>
					<div style={{ fontSize: "12px", color: "#9a9aa6" }}>Sora Lane</div>
					<div style={{ display: "flex", gap: "14px", alignItems: "center", marginTop: "12px" }}>
						<svg aria-hidden="true" fill="#fff" width="16" height="16" viewBox="0 0 24 24">
							<path d="M6 5h2v14H6zM20 5v14L9 12z" />
						</svg>
						<svg aria-hidden="true" fill="#fff" width="16" height="16" viewBox="0 0 24 24">
							<path d="M7 4h4v16H7zM13 4h4v16h-4z" />
						</svg>
						<svg aria-hidden="true" fill="#fff" width="16" height="16" viewBox="0 0 24 24">
							<path d="M16 5h2v14h-2zM4 5v14l11-7z" />
						</svg>
						<svg
							aria-hidden="true"
							width="18"
							height="18"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#30d158"
							strokeWidth="2"
							strokeLinecap="round"
						>
							<path d="M4 6h10M4 12h16M4 18h12" />
						</svg>
					</div>
				</div>
				<div
					style={{
						flex: "1",
						minWidth: "0",
						display: "flex",
						flexDirection: "column",
						gap: "10px",
					}}
				>
					<span
						style={{
							fontSize: "13px",
							color: "rgba(255,255,255,0.2)",
							whiteSpace: "nowrap",
							overflow: "hidden",
							textOverflow: "ellipsis",
						}}
					>
						Count the lights along the road
					</span>
					<span
						style={{
							fontSize: "17px",
							fontWeight: "500",
							whiteSpace: "nowrap",
							overflow: "hidden",
							textOverflow: "ellipsis",
						}}
					>
						We were never far from home
					</span>
					<span
						style={{
							fontSize: "13px",
							color: "rgba(255,255,255,0.2)",
							whiteSpace: "nowrap",
							overflow: "hidden",
							textOverflow: "ellipsis",
						}}
					>
						Leave the window open tonight
					</span>
				</div>
			</div>
		</OpenNotch>
	);
}
