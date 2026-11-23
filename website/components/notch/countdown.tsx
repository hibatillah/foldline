import { OpenNotch } from "./open-notch";

/** A page a contributor could add with the scripts in the Extend section: a countdown to a date. */
export function CountdownNotch() {
	return (
		<OpenNotch notch="countdown" activePage={3}>
			<div className="h-[92px] flex items-center gap-[24px]">
				<div className="flex-1 min-w-0 flex flex-col gap-[4px]">
					<span className="text-[12px] text-[#9a9aa6]">Counting down to</span>
					<span className="text-[20px] font-medium tracking-[-0.01em]">Release day</span>
					<div className="mt-[10px] h-[6px] rounded-[3px] bg-[#2a2a30] overflow-hidden">
						<div className="w-[62%] h-full rounded-[3px] bg-[#4aa3ff]" />
					</div>
				</div>
				<div className="flex-none flex items-baseline gap-[6px]">
					<span className="text-[52px] font-medium tracking-[-0.03em] leading-[1] tabular-nums">
						12
					</span>
					<span className="text-[14px] text-[#9a9aa6]">days</span>
				</div>
			</div>
		</OpenNotch>
	);
}
