export function TodoList() {
	return (
		<div
			style={{
				width: "100%",
				maxWidth: "240px",
				display: "flex",
				flexDirection: "column",
				gap: "6px",
			}}
		>
			<div
				style={{
					height: "32px",
					borderRadius: "10px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					display: "flex",
					alignItems: "center",
					gap: "10px",
					padding: "0 10px",
					fontSize: "13px",
					color: "#18181b",
					whiteSpace: "nowrap",
					overflow: "hidden",
				}}
			>
				<span
					style={{
						width: "12px",
						height: "12px",
						flex: "none",
						borderRadius: "50%",
						border: "1.5px solid #a1a1aa",
					}}
				/>
				Write release notes
			</div>
			<div
				style={{
					height: "32px",
					borderRadius: "10px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					display: "flex",
					alignItems: "center",
					gap: "10px",
					padding: "0 10px",
					fontSize: "13px",
					color: "#18181b",
					whiteSpace: "nowrap",
					overflow: "hidden",
				}}
			>
				<span
					style={{
						width: "12px",
						height: "12px",
						flex: "none",
						borderRadius: "50%",
						border: "1.5px solid #a1a1aa",
					}}
				/>
				Reply to the review
			</div>
			<div
				style={{
					height: "32px",
					borderRadius: "10px",
					background: "#ffffff",
					boxShadow: "0 0 0 1px #e4e4e7",
					display: "flex",
					alignItems: "center",
					gap: "10px",
					padding: "0 10px",
					fontSize: "13px",
					color: "#18181b",
					whiteSpace: "nowrap",
					overflow: "hidden",
				}}
			>
				<span
					style={{
						width: "12px",
						height: "12px",
						flex: "none",
						borderRadius: "50%",
						border: "1.5px solid #a1a1aa",
					}}
				/>
				Order a charger
			</div>
			<div
				style={{
					height: "32px",
					display: "flex",
					alignItems: "center",
					gap: "10px",
					padding: "0 10px",
					fontSize: "13px",
					color: "#a1a1aa",
					textDecoration: "line-through",
				}}
			>
				<svg
					aria-hidden="true"
					width="14"
					height="14"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#2f9e44"
					strokeWidth="3"
					strokeLinecap="round"
					strokeLinejoin="round"
				>
					<path d="M20 6 9 17l-5-5" />
				</svg>
				Back up the photos
			</div>
		</div>
	);
}
