import { Banner } from "./banner";

export function DownloadBanner() {
	return (
		<Banner width={340}>
			<div style={{ display: "flex", gap: "12px", alignItems: "flex-start" }}>
				<div
					style={{
						flex: "none",
						width: "30px",
						height: "30px",
						display: "flex",
						alignItems: "center",
						justifyContent: "center",
					}}
				>
					<div
						style={{
							width: "24px",
							height: "30px",
							borderRadius: "3px",
							background: "#fff",
							position: "relative",
						}}
					>
						<span
							style={{
								position: "absolute",
								left: "-3px",
								bottom: "6px",
								padding: "1px 3px",
								borderRadius: "2px",
								background: "#e03131",
								color: "#fff",
								fontSize: "8px",
								fontWeight: "500",
							}}
						>
							PDF
						</span>
					</div>
				</div>
				<div
					style={{ display: "flex", flexDirection: "column", gap: "2px", minWidth: "0", flex: "1" }}
				>
					<div
						style={{
							fontSize: "14px",
							fontWeight: "500",
							whiteSpace: "nowrap",
							overflow: "hidden",
							textOverflow: "ellipsis",
						}}
					>
						Annual report 2026.pdf
					</div>
					<div style={{ fontSize: "13px", color: "#adadb8" }}>Downloaded · 17.8 MB</div>
					<div style={{ alignSelf: "flex-end", marginTop: "10px", display: "flex", gap: "8px" }}>
						<span
							style={{
								height: "28px",
								padding: "0 14px",
								borderRadius: "14px",
								background: "rgba(255,255,255,0.14)",
								color: "#fff",
								fontSize: "13px",
								fontWeight: "500",
								display: "flex",
								alignItems: "center",
							}}
						>
							Show in folder
						</span>
						<span
							style={{
								height: "28px",
								padding: "0 14px",
								borderRadius: "14px",
								background: "#fff",
								color: "#000",
								fontSize: "13px",
								fontWeight: "500",
								display: "flex",
								alignItems: "center",
							}}
						>
							Open
						</span>
					</div>
				</div>
			</div>
		</Banner>
	);
}
