import Image from "next/image";
import { BatteryLow, Calendar, CloudRain, Copy, Folder, MoonStar, Volume2 } from "lucide-react";
import type { ReactNode } from "react";

import { Flares } from "./open-notch";
import { BatteryRingIcon, TimerRingIcon, WaveBars } from "./icons";

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
		<RestingPill>
			<TimerRingIcon />
			<BatteryRingIcon />
		</RestingPill>
	);
}

/** The pill at rest with something to show: the clock at the left, what a page or an activity shows at the right. */
function RestingPill({ children }: { children: ReactNode }) {
	return (
		<Pill className="min-w-[220px] justify-between! gap-[24px]! px-[12px]">
			<span className="text-[14px] tabular-nums">15:05</span>
			<span className="flex items-center gap-[8px]">{children}</span>
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

const ACCENT_BLUE = "#4dabf7";
const ACCENT_BLUE_LIGHT = "#74c0fc";
const MINT_GREEN = "#63e6be";
const LOW_RED = "#ff453a";

export function TimerPill() {
	return (
		<RestingPill>
			<TimerRingIcon />
			<BatteryRingIcon />
		</RestingPill>
	);
}

export function WavePill() {
	return (
		<RestingPill>
			<WaveBars />
			<BatteryRingIcon />
		</RestingPill>
	);
}

/** The media page's pill while lyrics show: the artwork at the left and the line being sung, with no clock. */
export function LyricsPill() {
	return (
		<Pill className="gap-[12px] px-[14px]">
			<Image
				src="/artwork/stuck-with-u.jpg"
				alt=""
				width={20}
				height={20}
				unoptimized
				className="block size-5 rounded-[5px]"
			/>
			<span>Wherever you are, wherever you go</span>
		</Pill>
	);
}

export function PrayerPill() {
	return (
		<RestingPill>
			<MoonStar aria-hidden="true" size={14} strokeWidth={2} color={MINT_GREEN} />
			<span className="text-[13px] tabular-nums" style={{ color: MINT_GREEN }}>
				Asr 14:52
			</span>
		</RestingPill>
	);
}

export function EventPill() {
	return (
		<RestingPill>
			<Calendar aria-hidden="true" size={14} strokeWidth={2} />
			<span>Standup</span>
			<span className="tabular-nums" style={{ color: ACCENT_BLUE }}>
				12 min
			</span>
		</RestingPill>
	);
}

export function CopiedPill() {
	return (
		<RestingPill>
			<Copy aria-hidden="true" size={14} strokeWidth={2} />
			<span>Copied</span>
		</RestingPill>
	);
}

export function ShelfPill() {
	return (
		<RestingPill>
			<Folder aria-hidden="true" size={14} strokeWidth={2} />
			<span>Added to shelf</span>
		</RestingPill>
	);
}

export function DeviceBatteryPill() {
	return (
		<RestingPill>
			<BatteryLow aria-hidden="true" size={14} strokeWidth={2} color={LOW_RED} />
			<span>Earbuds</span>
			<span style={{ color: LOW_RED }}>20%</span>
		</RestingPill>
	);
}

export function RainPill() {
	return (
		<RestingPill>
			<CloudRain aria-hidden="true" size={14} strokeWidth={2} color={ACCENT_BLUE_LIGHT} />
			<span style={{ color: ACCENT_BLUE_LIGHT }}>Rain in 30 min</span>
		</RestingPill>
	);
}
