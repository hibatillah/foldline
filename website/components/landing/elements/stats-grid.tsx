export function StatsGrid() {
	return (
		<div className="grid grid-cols-[repeat(2,_132px)] gap-[8px]">
			<div className="relative w-[132px] h-[86px] rounded-[12px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] text-[#18181b]">
				<span className="absolute left-[12px] top-[8px] text-[11px] text-[#71717a]">CPU</span>
				<svg
					aria-hidden="true"
					width="132"
					height="86"
					viewBox="0 0 132 86"
					className="absolute inset-0"
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
				<span className="absolute left-[12px] top-[22px] text-[20px] font-medium">12%</span>
			</div>
			<div className="relative w-[132px] h-[86px] rounded-[12px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] text-[#18181b]">
				<span className="absolute left-[12px] top-[8px] text-[11px] text-[#71717a]">Memory</span>
				<svg
					aria-hidden="true"
					width="132"
					height="86"
					viewBox="0 0 132 86"
					className="absolute inset-0"
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
				<span className="absolute left-[12px] top-[22px] text-[20px] font-medium">61%</span>
			</div>
			<div className="relative w-[132px] h-[86px] rounded-[12px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] text-[#18181b]">
				<span className="absolute left-[12px] top-[8px] text-[11px] text-[#71717a]">GPU</span>
				<svg
					aria-hidden="true"
					width="132"
					height="86"
					viewBox="0 0 132 86"
					className="absolute inset-0"
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
				<span className="absolute left-[12px] top-[22px] text-[20px] font-medium">8%</span>
			</div>
			<div className="relative w-[132px] h-[86px] rounded-[12px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] text-[#18181b]">
				<span className="absolute left-[12px] top-[8px] text-[11px] text-[#71717a]">Network</span>
				<span className="absolute left-[12px] top-[24px] text-[14px] font-medium">↓ 2.4 MB/s</span>
				<span className="absolute left-[12px] top-[46px] text-[13px] text-[#71717a]">
					↑ 180 KB/s
				</span>
				<span className="absolute left-[12px] top-[66px] text-[11px] text-[#71717a]">Wi-Fi</span>
			</div>
		</div>
	);
}
