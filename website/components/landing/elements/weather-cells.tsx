export function WeatherCells() {
	return (
		<div className="grid grid-cols-[repeat(3,_62px)] gap-[6px]">
			<div className="h-[86px] rounded-[10px] bg-[#fff4e0] [box-shadow:0_0_0_1px_#ffe0a8] flex flex-col items-center justify-center gap-[4px] text-[#18181b]">
				<span className="text-[11px] text-[#b5651d]">Now</span>
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
				<span className="text-[13px]">31°</span>
				<span className="text-[11px] text-[#1c7ed6] h-[13px]" />
			</div>
			<div className="h-[86px] rounded-[10px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] flex flex-col items-center justify-center gap-[4px] text-[#18181b]">
				<span className="text-[11px] text-[#71717a]">16:00</span>
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
				<span className="text-[13px]">29°</span>
				<span className="text-[11px] text-[#1c7ed6] h-[13px]">1.2</span>
			</div>
			<div className="h-[86px] rounded-[10px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] flex flex-col items-center justify-center gap-[4px] text-[#18181b]">
				<span className="text-[11px] text-[#71717a]">17:00</span>
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
				<span className="text-[13px]">27°</span>
				<span className="text-[11px] text-[#1c7ed6] h-[13px]">3.4</span>
			</div>
			<div className="h-[86px] rounded-[10px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] flex flex-col items-center justify-center gap-[4px] text-[#18181b]">
				<span className="text-[11px] text-[#71717a]">18:00</span>
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
				<span className="text-[13px]">27°</span>
				<span className="text-[11px] text-[#1c7ed6] h-[13px]">0.4</span>
			</div>
			<div className="h-[86px] rounded-[10px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] flex flex-col items-center justify-center gap-[4px] text-[#18181b]">
				<span className="text-[11px] text-[#71717a]">19:00</span>
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
				<span className="text-[13px]">26°</span>
				<span className="text-[11px] text-[#1c7ed6] h-[13px]" />
			</div>
			<div className="h-[86px] rounded-[10px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] flex flex-col items-center justify-center gap-[4px] text-[#18181b]">
				<span className="text-[11px] text-[#71717a]">20:00</span>
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
				<span className="text-[13px]">26°</span>
				<span className="text-[11px] text-[#1c7ed6] h-[13px]" />
			</div>
		</div>
	);
}
