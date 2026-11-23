import { OpenNotch } from "./open-notch";

export function CalendarWeekNotch() {
	return (
		<OpenNotch notch="calendar" activePage={2}>
			<div className="h-[86px] flex gap-[16px] overflow-hidden">
				<div className="w-[182px] flex-none">
					<div className="flex items-baseline gap-[6px]">
						<span className="text-[20px] font-medium">November</span>
						<span className="text-[12px] text-[#9a9aa6]">2026</span>
					</div>
					<div className="grid grid-cols-[repeat(7,_26px)] mt-[8px] text-[10px] text-[#9a9aa6] text-center">
						<span>M</span>
						<span>T</span>
						<span>W</span>
						<span>T</span>
						<span>F</span>
						<span>S</span>
						<span>S</span>
					</div>
					<div className="grid grid-cols-[repeat(7,_26px)] mt-[6px]">
						<span className="h-[20px] flex items-center justify-center">
							<span className="w-[20px] h-[20px] rounded-full text-[#e6e6eb] text-[11px] flex items-center justify-center">
								16
							</span>
						</span>
						<span className="h-[20px] flex items-center justify-center">
							<span className="w-[20px] h-[20px] rounded-full text-[#e6e6eb] text-[11px] flex items-center justify-center">
								17
							</span>
						</span>
						<span className="h-[20px] flex items-center justify-center">
							<span className="w-[20px] h-[20px] rounded-full text-[#e6e6eb] text-[11px] flex items-center justify-center">
								18
							</span>
						</span>
						<span className="h-[20px] flex items-center justify-center">
							<span className="w-[20px] h-[20px] rounded-full text-[#e6e6eb] text-[11px] flex items-center justify-center">
								19
							</span>
						</span>
						<span className="h-[20px] flex items-center justify-center">
							<span className="w-[20px] h-[20px] rounded-full text-[#e6e6eb] text-[11px] flex items-center justify-center">
								20
							</span>
						</span>
						<span className="h-[20px] flex items-center justify-center">
							<span className="w-[20px] h-[20px] rounded-full text-[#e6e6eb] text-[11px] flex items-center justify-center">
								21
							</span>
						</span>
						<span className="h-[20px] flex items-center justify-center">
							<span className="w-[20px] h-[20px] rounded-full bg-[#2f81f7] font-medium text-[#fff] text-[11px] flex items-center justify-center">
								22
							</span>
						</span>
					</div>
				</div>
				<div className="flex-1 flex flex-col gap-[4px] min-w-0">
					<div className="h-[26px] flex-none rounded-[7px] bg-[#17171c] [border:1px_solid_#26262d] flex items-center p-[0_9px] box-border text-[12px]">
						<span className="w-[44px] flex-none text-[11px] text-[#9a9aa6]">13:00</span>
						<span className="whitespace-nowrap overflow-hidden text-ellipsis">Release review</span>
					</div>
					<div className="h-[26px] flex-none rounded-[7px] bg-[#17171c] [border:1px_solid_#26262d] flex items-center p-[0_9px] box-border text-[12px]">
						<span className="w-[44px] flex-none text-[11px] text-[#9a9aa6]">15:30</span>
						<span className="whitespace-nowrap overflow-hidden text-ellipsis">Design sync</span>
					</div>
					<div className="h-[26px] flex-none rounded-[7px] bg-[#17171c] [border:1px_solid_#26262d] flex items-center p-[0_9px] box-border text-[12px]">
						<span className="w-[44px] flex-none text-[11px] text-[#9a9aa6]">19:00</span>
						<span className="whitespace-nowrap overflow-hidden text-ellipsis">Futsal</span>
					</div>
				</div>
			</div>
		</OpenNotch>
	);
}
