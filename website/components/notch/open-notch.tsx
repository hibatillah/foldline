import type { ReactNode } from "react";

import { BatteryRingIcon, GearIcon, MoonIcon, PinIcon, TimerRingIcon, WaveBars } from "./icons";
import { NOTCH_WIDTH_CLASS, type NotchKey } from "./sizes";

const FLARE = {
	6: {
		left: "-left-1.5 size-1.5 [background:radial-gradient(circle_6px_at_0_100%,transparent_98%,#000_100%)]",
		right:
			"-right-1.5 size-1.5 [background:radial-gradient(circle_6px_at_100%_100%,transparent_98%,#000_100%)]",
	},
	12: {
		left: "-left-3 size-3 [background:radial-gradient(circle_12px_at_0_100%,transparent_98%,#000_100%)]",
		right:
			"-right-3 size-3 [background:radial-gradient(circle_12px_at_100%_100%,transparent_98%,#000_100%)]",
	},
} as const;

/** The two rounded corners that flare out beside a notch. */
export function Flares({ size }: { size: 6 | 12 }) {
	return (
		<>
			<span aria-hidden="true" className={`absolute top-0 ${FLARE[size].left}`} />
			<span aria-hidden="true" className={`absolute top-0 ${FLARE[size].right}`} />
		</>
	);
}

/** The status marks of a usual notch: timer, prayer and battery. */
export function DefaultStatus() {
	return (
		<>
			<TimerRingIcon />
			<MoonIcon />
			<BatteryRingIcon />
		</>
	);
}

/** The status marks while music plays. */
export function MediaStatus() {
	return (
		<>
			<WaveBars />
			<TimerRingIcon />
			<BatteryRingIcon />
		</>
	);
}

const PAGE_COUNT = 10;

function NotchHeader({ activePage, status }: { activePage: number; status: ReactNode }) {
	return (
		<div className="relative h-[16px] flex items-center justify-between">
			<div className="flex items-center gap-[4px]">
				{Array.from({ length: PAGE_COUNT }, (_, index) =>
					index === activePage ? (
						<span key={index} className="w-[16px] h-[6px] rounded-[3px] bg-[#e6e6eb]" />
					) : (
						<span key={index} className="w-[6px] h-[6px] rounded-[3px] bg-[#3a3a44]" />
					),
				)}
			</div>
			<span className="absolute left-[50%] top-[50%] [transform:translate(-50%,_-50%)] text-[14px] font-medium leading-[16px] tabular-nums">
				15:05
			</span>
			<div className="flex items-center gap-[8px]">
				{status}
				<span className="flex items-center gap-[4px]">
					<PinIcon />
					<GearIcon />
				</span>
			</div>
		</div>
	);
}

/** The opened notch: black shape with flares, a page header and the page content. */
export function OpenNotch({
	notch,
	activePage,
	background = "bg-black",
	status,
	children,
}: {
	notch: NotchKey;
	activePage: number;
	/** A background class; plain black unless the page tints its notch. */
	background?: string;
	status?: ReactNode;
	children: ReactNode;
}) {
	return (
		<div
			className={`relative box-border flex flex-col gap-2.5 rounded-[0_0_22px_22px] p-[10px_16px_14px] text-left font-sans text-white ${NOTCH_WIDTH_CLASS[notch]} ${background}`}
		>
			<Flares size={12} />
			<NotchHeader activePage={activePage} status={status ?? <DefaultStatus />} />
			{children}
		</div>
	);
}
