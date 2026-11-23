import { OpenNotch } from "./open-notch";

export function CalendarWeekNotch() {
	return (
		<OpenNotch width={440} activePage={2}>
			<div style={{ height: "86px", display: "flex", gap: "16px", overflow: "hidden" }}>
				<div style={{ width: "182px", flex: "none" }}>
					<div style={{ display: "flex", alignItems: "baseline", gap: "6px" }}>
						<span style={{ fontSize: "20px", fontWeight: "500" }}>November</span>
						<span style={{ fontSize: "12px", color: "#9a9aa6" }}>2026</span>
					</div>
					<div
						style={{
							display: "grid",
							gridTemplateColumns: "repeat(7, 26px)",
							marginTop: "8px",
							fontSize: "10px",
							color: "#9a9aa6",
							textAlign: "center",
						}}
					>
						<span>M</span>
						<span>T</span>
						<span>W</span>
						<span>T</span>
						<span>F</span>
						<span>S</span>
						<span>S</span>
					</div>
					<div
						style={{ display: "grid", gridTemplateColumns: "repeat(7, 26px)", marginTop: "6px" }}
					>
						<span
							style={{
								height: "20px",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
							}}
						>
							<span
								style={{
									width: "20px",
									height: "20px",
									borderRadius: "50%",
									color: "#e6e6eb",
									fontSize: "11px",
									display: "flex",
									alignItems: "center",
									justifyContent: "center",
								}}
							>
								16
							</span>
						</span>
						<span
							style={{
								height: "20px",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
							}}
						>
							<span
								style={{
									width: "20px",
									height: "20px",
									borderRadius: "50%",
									color: "#e6e6eb",
									fontSize: "11px",
									display: "flex",
									alignItems: "center",
									justifyContent: "center",
								}}
							>
								17
							</span>
						</span>
						<span
							style={{
								height: "20px",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
							}}
						>
							<span
								style={{
									width: "20px",
									height: "20px",
									borderRadius: "50%",
									color: "#e6e6eb",
									fontSize: "11px",
									display: "flex",
									alignItems: "center",
									justifyContent: "center",
								}}
							>
								18
							</span>
						</span>
						<span
							style={{
								height: "20px",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
							}}
						>
							<span
								style={{
									width: "20px",
									height: "20px",
									borderRadius: "50%",
									color: "#e6e6eb",
									fontSize: "11px",
									display: "flex",
									alignItems: "center",
									justifyContent: "center",
								}}
							>
								19
							</span>
						</span>
						<span
							style={{
								height: "20px",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
							}}
						>
							<span
								style={{
									width: "20px",
									height: "20px",
									borderRadius: "50%",
									color: "#e6e6eb",
									fontSize: "11px",
									display: "flex",
									alignItems: "center",
									justifyContent: "center",
								}}
							>
								20
							</span>
						</span>
						<span
							style={{
								height: "20px",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
							}}
						>
							<span
								style={{
									width: "20px",
									height: "20px",
									borderRadius: "50%",
									color: "#e6e6eb",
									fontSize: "11px",
									display: "flex",
									alignItems: "center",
									justifyContent: "center",
								}}
							>
								21
							</span>
						</span>
						<span
							style={{
								height: "20px",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
							}}
						>
							<span
								style={{
									width: "20px",
									height: "20px",
									borderRadius: "50%",
									background: "#2f81f7",
									fontWeight: "500",
									color: "#fff",
									fontSize: "11px",
									display: "flex",
									alignItems: "center",
									justifyContent: "center",
								}}
							>
								22
							</span>
						</span>
					</div>
				</div>
				<div
					style={{ flex: "1", display: "flex", flexDirection: "column", gap: "4px", minWidth: "0" }}
				>
					<div
						style={{
							height: "26px",
							flex: "none",
							borderRadius: "7px",
							background: "#17171c",
							border: "1px solid #26262d",
							display: "flex",
							alignItems: "center",
							padding: "0 9px",
							boxSizing: "border-box",
							fontSize: "12px",
						}}
					>
						<span style={{ width: "44px", flex: "none", fontSize: "11px", color: "#9a9aa6" }}>
							13:00
						</span>
						<span style={{ whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
							Release review
						</span>
					</div>
					<div
						style={{
							height: "26px",
							flex: "none",
							borderRadius: "7px",
							background: "#17171c",
							border: "1px solid #26262d",
							display: "flex",
							alignItems: "center",
							padding: "0 9px",
							boxSizing: "border-box",
							fontSize: "12px",
						}}
					>
						<span style={{ width: "44px", flex: "none", fontSize: "11px", color: "#9a9aa6" }}>
							15:30
						</span>
						<span style={{ whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
							Design sync
						</span>
					</div>
					<div
						style={{
							height: "26px",
							flex: "none",
							borderRadius: "7px",
							background: "#17171c",
							border: "1px solid #26262d",
							display: "flex",
							alignItems: "center",
							padding: "0 9px",
							boxSizing: "border-box",
							fontSize: "12px",
						}}
					>
						<span style={{ width: "44px", flex: "none", fontSize: "11px", color: "#9a9aa6" }}>
							19:00
						</span>
						<span style={{ whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
							Futsal
						</span>
					</div>
				</div>
			</div>
		</OpenNotch>
	);
}
