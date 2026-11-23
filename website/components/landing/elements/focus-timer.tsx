export function FocusTimer() {
	return (
		<div className="w-full max-w-[300px] text-[#18181b]">
			<div className="flex items-end gap-[12px]">
				<span className="text-[44px] font-medium leading-[46px] tracking-[-0.01em] tabular-nums">
					25:00
				</span>
				<span className="flex flex-col pb-[6px]">
					<span className="text-[14px] font-medium">Focus</span>
					<span className="text-[12px] text-[#71717a]">Session 1 of 4</span>
				</span>
			</div>
			<div className="mt-[16px] grid grid-cols-[repeat(2,_minmax(0,_1fr))] gap-[9px]">
				<span className="h-[34px] rounded-[17px] bg-[#ff9e0a] text-[#1a1205] text-[13px] font-medium flex items-center justify-center">
					Start
				</span>
				<span className="h-[34px] rounded-[17px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] text-[13px] font-medium flex items-center justify-center">
					Short break
				</span>
			</div>
		</div>
	);
}
