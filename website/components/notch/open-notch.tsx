import type { ReactNode } from "react";

import { BatteryRingIcon, GearIcon, MoonIcon, PinIcon, TimerRingIcon, WaveBars } from "./icons";

/** The two rounded corners that flare out beside a notch. */
export function Flares({ size }: { size: number }) {
	return (
		<>
			<span
				aria-hidden="true"
				style={{
					position: "absolute",
					left: -size,
					top: 0,
					width: size,
					height: size,
					background: `radial-gradient(circle ${size}px at 0 100%, transparent 98%, #000 100%)`,
				}}
			/>
			<span
				aria-hidden="true"
				style={{
					position: "absolute",
					right: -size,
					top: 0,
					width: size,
					height: size,
					background: `radial-gradient(circle ${size}px at 100% 100%, transparent 98%, #000 100%)`,
				}}
			/>
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
		<div
			style={{
				position: "relative",
				height: 16,
				display: "flex",
				alignItems: "center",
				justifyContent: "space-between",
			}}
		>
			<div style={{ display: "flex", alignItems: "center", gap: 4 }}>
				{Array.from({ length: PAGE_COUNT }, (_, index) =>
					index === activePage ? (
						<span
							key={index}
							style={{ width: 16, height: 6, borderRadius: 3, background: "#e6e6eb" }}
						/>
					) : (
						<span
							key={index}
							style={{ width: 6, height: 6, borderRadius: 3, background: "#3a3a44" }}
						/>
					),
				)}
			</div>
			<span
				style={{
					position: "absolute",
					left: "50%",
					top: "50%",
					transform: "translate(-50%, -50%)",
					fontSize: 14,
					fontWeight: 500,
					lineHeight: "16px",
					fontVariantNumeric: "tabular-nums",
				}}
			>
				15:05
			</span>
			<div style={{ display: "flex", alignItems: "center", gap: 8 }}>
				{status}
				<span style={{ display: "flex", alignItems: "center", gap: 4 }}>
					<PinIcon />
					<GearIcon />
				</span>
			</div>
		</div>
	);
}

/** The opened notch: black shape with flares, a page header and the page content. */
export function OpenNotch({
	width,
	activePage,
	background = "#000",
	status,
	children,
}: {
	width: number;
	activePage: number;
	background?: string;
	status?: ReactNode;
	children: ReactNode;
}) {
	return (
		<div
			style={{
				position: "relative",
				width,
				background,
				borderRadius: "0 0 22px 22px",
				padding: "10px 16px 14px",
				boxSizing: "border-box",
				display: "flex",
				flexDirection: "column",
				gap: 10,
				color: "#fff",
				fontFamily: "var(--font-sans)",
				textAlign: "left",
			}}
		>
			<Flares size={12} />
			<NotchHeader activePage={activePage} status={status ?? <DefaultStatus />} />
			{children}
		</div>
	);
}
