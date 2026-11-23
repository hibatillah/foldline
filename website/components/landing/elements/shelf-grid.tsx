export function ShelfGrid() {
	return (
		<div style={{ border: "1.5px dashed #a1a1aa", borderRadius: "14px", padding: "14px 10px" }}>
			<div style={{ display: "grid", gridTemplateColumns: "repeat(3, 72px)", gap: "12px 6px" }}>
				<div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "4px" }}>
					<div
						style={{
							width: "40px",
							height: "40px",
							display: "flex",
							alignItems: "center",
							justifyContent: "center",
							filter: "drop-shadow(0 1px 1.5px rgba(0,0,0,0.18))",
						}}
					>
						<div
							style={{
								width: "30px",
								height: "38px",
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
					<span
						style={{
							fontSize: "11px",
							color: "#71717a",
							maxWidth: "70px",
							whiteSpace: "nowrap",
							overflow: "hidden",
							textOverflow: "ellipsis",
						}}
					>
						Brief v3.pdf
					</span>
				</div>
				<div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "4px" }}>
					<div
						style={{
							width: "40px",
							height: "40px",
							display: "flex",
							alignItems: "center",
							justifyContent: "center",
							filter: "drop-shadow(0 1px 1.5px rgba(0,0,0,0.18))",
						}}
					>
						<div
							style={{
								width: "40px",
								height: "40px",
								borderRadius: "6px",
								background: "linear-gradient(160deg, #a5d8ff, #4c6ef5 60%, #364fc7)",
							}}
						/>
					</div>
					<span
						style={{
							fontSize: "11px",
							color: "#71717a",
							maxWidth: "70px",
							whiteSpace: "nowrap",
							overflow: "hidden",
							textOverflow: "ellipsis",
						}}
					>
						hero-shot.png
					</span>
				</div>
				<div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "4px" }}>
					<div
						style={{
							width: "40px",
							height: "40px",
							display: "flex",
							alignItems: "center",
							justifyContent: "center",
							filter: "drop-shadow(0 1px 1.5px rgba(0,0,0,0.18))",
						}}
					>
						<div
							style={{
								width: "34px",
								height: "28px",
								borderRadius: "4px",
								background: "#f8bd3b",
								position: "relative",
							}}
						>
							<span
								style={{
									position: "absolute",
									left: "15px",
									top: "4px",
									width: "4px",
									height: "18px",
									background:
										"repeating-linear-gradient(180deg, #8a6514 0 2px, transparent 2px 4px)",
								}}
							/>
						</div>
					</div>
					<span
						style={{
							fontSize: "11px",
							color: "#71717a",
							maxWidth: "70px",
							whiteSpace: "nowrap",
							overflow: "hidden",
							textOverflow: "ellipsis",
						}}
					>
						assets.zip
					</span>
				</div>
				<div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "4px" }}>
					<div
						style={{
							width: "40px",
							height: "40px",
							display: "flex",
							alignItems: "center",
							justifyContent: "center",
							filter: "drop-shadow(0 1px 1.5px rgba(0,0,0,0.18))",
						}}
					>
						<div
							style={{
								width: "30px",
								height: "38px",
								borderRadius: "3px",
								background: "#fff",
								display: "flex",
								flexDirection: "column",
								gap: "4px",
								padding: "8px 5px",
								boxSizing: "border-box",
							}}
						>
							<span style={{ height: "1px", background: "#c4c4cc" }} />
							<span style={{ height: "1px", background: "#c4c4cc" }} />
							<span style={{ height: "1px", background: "#c4c4cc" }} />
							<span style={{ height: "1px", width: "60%", background: "#c4c4cc" }} />
						</div>
					</div>
					<span
						style={{
							fontSize: "11px",
							color: "#71717a",
							maxWidth: "70px",
							whiteSpace: "nowrap",
							overflow: "hidden",
							textOverflow: "ellipsis",
						}}
					>
						notes.txt
					</span>
				</div>
				<div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "4px" }}>
					<div
						style={{
							width: "40px",
							height: "40px",
							display: "flex",
							alignItems: "center",
							justifyContent: "center",
							filter: "drop-shadow(0 1px 1.5px rgba(0,0,0,0.18))",
						}}
					>
						<div
							style={{
								width: "30px",
								height: "38px",
								borderRadius: "3px",
								background: "#fff",
								position: "relative",
								overflow: "hidden",
							}}
						>
							<span
								style={{
									position: "absolute",
									left: "0",
									top: "0",
									bottom: "0",
									width: "7px",
									background: "#2f9e44",
								}}
							/>
							<span
								style={{
									position: "absolute",
									right: "-2px",
									bottom: "4px",
									width: "14px",
									height: "14px",
									borderRadius: "3px",
									background: "#107c41",
									color: "#fff",
									fontSize: "9px",
									fontWeight: "500",
									display: "flex",
									alignItems: "center",
									justifyContent: "center",
								}}
							>
								X
							</span>
						</div>
					</div>
					<span
						style={{
							fontSize: "11px",
							color: "#71717a",
							maxWidth: "70px",
							whiteSpace: "nowrap",
							overflow: "hidden",
							textOverflow: "ellipsis",
						}}
					>
						Budget 2026.xlsx
					</span>
				</div>
				<div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "4px" }}>
					<div
						style={{
							width: "40px",
							height: "40px",
							display: "flex",
							alignItems: "center",
							justifyContent: "center",
							filter: "drop-shadow(0 1px 1.5px rgba(0,0,0,0.18))",
						}}
					>
						<div
							style={{
								width: "30px",
								height: "38px",
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
									background: "#3b5bdb",
									color: "#fff",
									fontSize: "8px",
									fontWeight: "500",
								}}
							>
								DOC
							</span>
						</div>
					</div>
					<span
						style={{
							fontSize: "11px",
							color: "#71717a",
							maxWidth: "70px",
							whiteSpace: "nowrap",
							overflow: "hidden",
							textOverflow: "ellipsis",
						}}
					>
						Contract.docx
					</span>
				</div>
			</div>
		</div>
	);
}
