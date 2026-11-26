import { Volume2 } from "lucide-react";
import type { ReactNode } from "react";

import { Flares } from "./open-notch";
import { BatteryRingIcon, MoonIcon, TimerRingIcon, WaveBars } from "./icons";

/** The collapsed notch: a black pill that hangs from the top edge, with a flare on each side. */
export function Pill({ children, className = "" }: { children?: ReactNode; className?: string }) {
	return (
		<div
			className={`relative box-border flex h-[34px] items-center justify-center gap-[10px] rounded-[0_0_18px_18px] bg-black px-[16px] text-left font-sans text-[13px] font-medium whitespace-nowrap text-white ${className}`}
		>
			<Flares size={6} />
			{children}
		</div>
	);
}

function Bar({ level }: { level: number }) {
	return (
		<span className="h-[5px] w-[96px] overflow-hidden rounded-[3px] bg-[#2a2a30]">
			<span className="block h-full rounded-[3px] bg-white" style={{ width: `${level}%` }} />
		</span>
	);
}

/** The pill at rest: the clock at the left edge and the marks of what lasts, such as a timer and the battery, at the right. */
export function ClockPill() {
	return (
		<Pill className="w-[220px] justify-between! px-[12px]">
			<span className="text-[14px] tabular-nums">15:05</span>
			<span className="flex items-center gap-[8px]">
				<TimerRingIcon />
				<BatteryRingIcon />
			</span>
		</Pill>
	);
}

/** A message: it takes over the pill for a moment, then goes. */
export function VolumePill() {
	return (
		<Pill className="gap-[12px] px-[18px]">
			<Volume2 aria-hidden="true" size={16} strokeWidth={2} />
			<Bar level={64} />
		</Pill>
	);
}

/** A badge: a small mark that stays at the right edge while something lasts. */
export function TimerPill() {
	return (
		<Pill className="w-[180px] justify-end">
			<TimerRingIcon />
			<BatteryRingIcon />
		</Pill>
	);
}

export function WavePill() {
	return (
		<Pill className="w-[180px] justify-end">
			<WaveBars />
			<BatteryRingIcon />
		</Pill>
	);
}

export function PrayerPill() {
	return (
		<Pill className="w-[180px] justify-end">
			<MoonIcon />
			<span className="text-[12px] tabular-nums text-[#adadb8]">Asr 14:52</span>
		</Pill>
	);
}

export function EventPill() {
	return (
		<Pill className="w-[180px] justify-end">
			<span className="text-[12px] tabular-nums text-[#adadb8]">Standup in 12 min</span>
		</Pill>
	);
}

export function CopiedPill() {
	return <Pill className="px-[18px]">Copied</Pill>;
}

export function ShelfPill() {
	return <Pill className="px-[18px]">Added to shelf</Pill>;
}

export function DeviceBatteryPill() {
	return (
		<Pill className="gap-[8px] px-[18px]">
			<span>Earbuds</span>
			<span className="text-[#ff453a]">Low battery 20%</span>
		</Pill>
	);
}

export function RainPill() {
	return <Pill className="px-[18px]">Rain in 30 min</Pill>;
}

export function TimerDonePill() {
	return <Pill className="px-[18px]">Focus round done</Pill>;
}
