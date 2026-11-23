export function VolumeSliders() {
	return (
		<div className="w-full max-w-[240px] flex flex-col gap-[24px]">
			<div className="flex items-center gap-[12px] text-[13px] font-medium text-[#18181b]">
				<svg
					aria-hidden="true"
					width="16"
					height="16"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#18181b"
					strokeWidth="2"
				>
					<path d="M11 5 6 9H3v6h3l5 4z" />
					<path d="M15.5 8.5a5 5 0 0 1 0 7M18.5 5.5a9 9 0 0 1 0 13" />
				</svg>
				<span className="flex-1 h-[4px] rounded-[2px] bg-[#e4e4e7]">
					<span className="block w-[42%] h-[4px] rounded-[2px] bg-[#18181b]" />
				</span>
				<span className="w-[18px] tabular-nums">42</span>
			</div>
			<div className="flex items-center gap-[12px] text-[13px] font-medium text-[#18181b]">
				<svg
					aria-hidden="true"
					width="16"
					height="16"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#18181b"
					strokeWidth="2"
				>
					<path d="M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8" />
					<path d="M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
				</svg>
				<span className="flex-1 h-[4px] rounded-[2px] bg-[#e4e4e7]">
					<span className="block w-[70%] h-[4px] rounded-[2px] bg-[#18181b]" />
				</span>
				<span className="w-[18px] tabular-nums">70</span>
			</div>
		</div>
	);
}
