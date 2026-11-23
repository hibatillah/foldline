export function GearIcon() {
	return (
		<svg
			aria-hidden="true"
			width="14"
			height="14"
			viewBox="0 0 24 24"
			fill="none"
			stroke="#8c8c94"
			strokeWidth="2"
			strokeLinejoin="round"
		>
			<path d="M12 2.5 20.2 7.25v9.5L12 21.5 3.8 16.75v-9.5z" />
			<circle cx="12" cy="12" r="3" />
		</svg>
	);
}

export function PinIcon() {
	return (
		<svg
			aria-hidden="true"
			width="13"
			height="13"
			viewBox="0 0 24 24"
			fill="none"
			stroke="#8c8c94"
			strokeWidth="2"
			strokeLinecap="round"
			strokeLinejoin="round"
		>
			<path d="M12 17v5" />
			<path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z" />
		</svg>
	);
}

export function TimerRingIcon() {
	return (
		<svg aria-hidden="true" width="14" height="14" viewBox="0 0 16 16">
			<circle cx="8" cy="8" r="6" fill="none" stroke="#38383a" strokeWidth="2" />
			<path
				d="M8 2 A6 6 0 1 1 2.8 11"
				fill="none"
				stroke="#ff9e0a"
				strokeWidth="2"
				strokeLinecap="round"
			/>
		</svg>
	);
}

export function MoonIcon() {
	return (
		<svg
			aria-hidden="true"
			width="14"
			height="14"
			viewBox="0 0 24 24"
			fill="none"
			stroke="#63e6be"
			strokeWidth="2"
			strokeLinecap="round"
			strokeLinejoin="round"
		>
			<path d="M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z" />
		</svg>
	);
}

export function BatteryRingIcon() {
	return (
		<svg aria-hidden="true" width="14" height="14" viewBox="0 0 16 16">
			<circle cx="8" cy="8" r="6" fill="none" stroke="#38383a" strokeWidth="2" />
			<path
				d="M8 2 A6 6 0 1 1 3.2 4.4"
				fill="none"
				stroke="#2f9e44"
				strokeWidth="2"
				strokeLinecap="round"
			/>
		</svg>
	);
}

export function WaveBars() {
	return (
		<span aria-hidden="true" className="flex items-center gap-[2px] w-[18px] h-[12px]">
			<span className="w-[2px] h-[5px] rounded-[1px] bg-[#fff]" />
			<span className="w-[2px] h-[10px] rounded-[1px] bg-[#fff]" />
			<span className="w-[2px] h-[12px] rounded-[1px] bg-[#fff]" />
			<span className="w-[2px] h-[8px] rounded-[1px] bg-[#fff]" />
			<span className="w-[2px] h-[4px] rounded-[1px] bg-[#fff]" />
		</span>
	);
}
