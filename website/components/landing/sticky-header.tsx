"use client";

import { useEffect, useRef, useState, type ReactNode } from "react";

/** The page header: flush at the top, and marked `data-stuck` once the page has scrolled so the CSS can float it. */
export function StickyHeader({ children }: { children: ReactNode }) {
	const sentinel = useRef<HTMLDivElement>(null);
	const [stuck, setStuck] = useState(false);

	useEffect(() => {
		const element = sentinel.current;
		if (!element) return;
		// The 1 px sentinel sits at the very top of the page, so it leaves the viewport on the first scroll.
		const observer = new IntersectionObserver(([entry]) => setStuck(!entry.isIntersecting));
		observer.observe(element);
		return () => observer.disconnect();
	}, []);

	return (
		<>
			<div
				ref={sentinel}
				aria-hidden="true"
				className="pointer-events-none absolute top-0 left-0 h-px w-px"
			/>
			<header
				className="group sticky top-0 z-20 h-[60px] p-0 transition-[padding] duration-250 data-[stuck=true]:px-[clamp(16px,3vw,40px)] motion-reduce:transition-none"
				data-stuck={stuck}
			>
				{children}
			</header>
		</>
	);
}
