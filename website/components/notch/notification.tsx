import { Banner } from "./banner";

export function WhatsAppBanner() {
	return (
		<Banner width={380}>
			<div style={{ display: "flex", gap: "12px" }}>
				<div
					aria-hidden="true"
					style={{
						flex: "none",
						width: "30px",
						height: "30px",
						borderRadius: "8px",
						background: "#25d366",
						display: "flex",
						alignItems: "center",
						justifyContent: "center",
					}}
				>
					<svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="#fff">
						<path d="M12 2a10 10 0 0 0-8.6 15.1L2 22l5-1.3A10 10 0 1 0 12 2zm0 1.8a8.2 8.2 0 1 1-4.2 15.3l-.3-.2-3 .8.8-2.9-.2-.3A8.2 8.2 0 0 1 12 3.8zm-3.3 4.4c-.2 0-.5 0-.7.3-.2.3-.9.9-.9 2.2s.9 2.5 1 2.7c.1.2 1.8 2.8 4.4 3.9 2.2.9 2.6.7 3.1.7.5 0 1.5-.6 1.7-1.2.2-.6.2-1.1.2-1.2l-.4-.3-1.7-.8c-.2-.1-.4-.1-.6.1l-.8 1c-.1.2-.3.2-.5.1-.3-.1-1-.4-2-1.2-.7-.6-1.2-1.4-1.3-1.7-.1-.2 0-.4.1-.5l.4-.5.3-.4v-.5l-.8-1.9c-.2-.5-.4-.4-.6-.4z" />
					</svg>
				</div>
				<div
					style={{ display: "flex", flexDirection: "column", gap: "2px", minWidth: "0", flex: "1" }}
				>
					<div style={{ fontSize: "14px", fontWeight: "500", display: "flex", gap: "8px" }}>
						<span>Nadia</span>
						<span style={{ color: "#8e8e99", fontWeight: "400" }}>WhatsApp</span>
					</div>
					<div
						style={{
							color: "#adadb8",
							fontSize: "13px",
							lineHeight: "17px",
							display: "-webkit-box",
							WebkitLineClamp: "2",
							WebkitBoxOrient: "vertical",
							overflow: "hidden",
						}}
					>
						Landed. Grabbing a coffee, then heading to the office. Want one?
					</div>
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
							Dismiss
						</span>
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
							Open
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
								gap: "6px",
							}}
						>
							<svg
								aria-hidden="true"
								width="12"
								height="12"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								strokeWidth="2.4"
								strokeLinecap="round"
								strokeLinejoin="round"
							>
								<path d="m9 17-5-5 5-5" />
								<path d="M20 18v-2a4 4 0 0 0-4-4H4" />
							</svg>
							Reply
						</span>
					</div>
				</div>
			</div>
		</Banner>
	);
}
