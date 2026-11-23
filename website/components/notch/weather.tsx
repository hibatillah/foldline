import { OpenNotch } from "./open-notch";

export function WeatherNotch() {
	return (
		<OpenNotch notch="weather" activePage={6}>
			<div className="h-[86px] flex gap-[16px]">
				<div className="w-[146px] flex-none flex flex-col">
					<div className="flex items-center justify-between">
						<span className="text-[10px] text-[#9a9aa6]">Pekanbaru</span>
						<span className="h-[22px] p-[0_9px] rounded-[11px] bg-[#1c1c22] [border:1px_solid_#2c2c34] box-border text-[11px] text-[#e6e6eb] flex items-center">
							Today
						</span>
					</div>
					<span className="text-[28px] font-medium leading-[32px]">31°</span>
					<span className="text-[11px] text-[#9a9aa6]">Partly cloudy · feels 35°</span>
					<span className="text-[12px] text-[#74c0fc] mt-[2px]">Rain from 16:00</span>
				</div>
				<div className="flex-1 grid grid-cols-[repeat(6,_minmax(0,_1fr))] gap-[4px]">
					<div className="rounded-[10px] bg-[#2b2312] flex flex-col items-center justify-center gap-[4px]">
						<span className="text-[10px] text-[#ffc078]">Now</span>
						<svg
							aria-hidden="true"
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#ffc078"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d="M12 2v2M4.9 4.9l1.4 1.4M20 12h2M19.1 4.9l-1.4 1.4M15.9 13.1A4 4 0 0 0 8.4 9.6" />
							<path d="M13 22H7a5 5 0 1 1 4.9-6H13a3 3 0 0 1 0 6z" />
						</svg>
						<span className="text-[13px]">31°</span>
						<span className="text-[10px] text-[#74c0fc] h-[12px]" />
					</div>
					<div className="rounded-[10px] flex flex-col items-center justify-center gap-[4px]">
						<span className="text-[10px] text-[#9a9aa6]">16:00</span>
						<svg
							aria-hidden="true"
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#74c0fc"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d="M4 14.9A7 7 0 1 1 15.7 8h1.8a4.5 4.5 0 0 1 2.5 8.2" />
							<path d="M16 14v6M8 14v6M12 16v6" />
						</svg>
						<span className="text-[13px]">29°</span>
						<span className="text-[10px] text-[#74c0fc] h-[12px]">1.2</span>
					</div>
					<div className="rounded-[10px] flex flex-col items-center justify-center gap-[4px]">
						<span className="text-[10px] text-[#9a9aa6]">17:00</span>
						<svg
							aria-hidden="true"
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#74c0fc"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d="M4 14.9A7 7 0 1 1 15.7 8h1.8a4.5 4.5 0 0 1 2.5 8.2" />
							<path d="M16 14v6M8 14v6M12 16v6" />
						</svg>
						<span className="text-[13px]">27°</span>
						<span className="text-[10px] text-[#74c0fc] h-[12px]">3.4</span>
					</div>
					<div className="rounded-[10px] flex flex-col items-center justify-center gap-[4px]">
						<span className="text-[10px] text-[#9a9aa6]">18:00</span>
						<svg
							aria-hidden="true"
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#c4c4cc"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d="M17.5 19H9a7 7 0 1 1 6.7-9h1.8a4.5 4.5 0 1 1 0 9z" />
						</svg>
						<span className="text-[13px]">27°</span>
						<span className="text-[10px] text-[#74c0fc] h-[12px]">0.4</span>
					</div>
					<div className="rounded-[10px] flex flex-col items-center justify-center gap-[4px]">
						<span className="text-[10px] text-[#9a9aa6]">19:00</span>
						<svg
							aria-hidden="true"
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#c4c4cc"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d="M17.5 19H9a7 7 0 1 1 6.7-9h1.8a4.5 4.5 0 1 1 0 9z" />
						</svg>
						<span className="text-[13px]">26°</span>
						<span className="text-[10px] text-[#74c0fc] h-[12px]" />
					</div>
					<div className="rounded-[10px] flex flex-col items-center justify-center gap-[4px]">
						<span className="text-[10px] text-[#9a9aa6]">20:00</span>
						<svg
							aria-hidden="true"
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#c4c4cc"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d="M17.5 19H9a7 7 0 1 1 6.7-9h1.8a4.5 4.5 0 1 1 0 9z" />
						</svg>
						<span className="text-[13px]">26°</span>
						<span className="text-[10px] text-[#74c0fc] h-[12px]" />
					</div>
				</div>
			</div>
		</OpenNotch>
	);
}
