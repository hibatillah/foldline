export function TodoList() {
	return (
		<div className="w-full max-w-[240px] flex flex-col gap-[6px]">
			<div className="h-[32px] rounded-[10px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] flex items-center gap-[10px] p-[0_10px] text-[13px] text-[#18181b] whitespace-nowrap overflow-hidden">
				<span className="w-[12px] h-[12px] flex-none rounded-full [border:1.5px_solid_#a1a1aa]" />
				Write release notes
			</div>
			<div className="h-[32px] rounded-[10px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] flex items-center gap-[10px] p-[0_10px] text-[13px] text-[#18181b] whitespace-nowrap overflow-hidden">
				<span className="w-[12px] h-[12px] flex-none rounded-full [border:1.5px_solid_#a1a1aa]" />
				Reply to the review
			</div>
			<div className="h-[32px] rounded-[10px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] flex items-center gap-[10px] p-[0_10px] text-[13px] text-[#18181b] whitespace-nowrap overflow-hidden">
				<span className="w-[12px] h-[12px] flex-none rounded-full [border:1.5px_solid_#a1a1aa]" />
				Order a charger
			</div>
			<div className="h-[32px] flex items-center gap-[10px] p-[0_10px] text-[13px] text-[#a1a1aa] [text-decoration:line-through]">
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
