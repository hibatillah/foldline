/** Thin blue wave lines behind a notch drawing, computed at build time from a seed. */
export function Waves({ seed, color = "37,99,235" }: { seed: number; color?: string }) {
	const paths = Array.from({ length: 46 }, (_, index) => {
		const offset = index * 9 - 40;
		const amplitude = 26 + ((index * 7 + seed * 13) % 18);
		const phase = seed * 0.9 + index * 0.06;
		let d = "";
		for (let x = -20; x <= 1320; x += 40) {
			const y =
				offset + amplitude * Math.sin(x / 210 + phase) + 18 * Math.sin(x / 90 + index * 0.3);
			d += `${d ? "L" : "M"}${x} ${y.toFixed(1)} `;
		}
		const alpha = (0.05 + 0.09 * Math.abs(Math.sin(index / 7 + seed))).toFixed(3);
		return { d, stroke: `rgba(${color},${alpha})` };
	});

	return (
		<svg
			aria-hidden="true"
			viewBox="0 0 1300 420"
			preserveAspectRatio="xMidYMid slice"
			className="absolute inset-0 w-full h-full"
		>
			{paths.map((path) => (
				<path key={path.d} d={path.d} fill="none" stroke={path.stroke} strokeWidth="1.1" />
			))}
		</svg>
	);
}
