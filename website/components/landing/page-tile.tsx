"use client";

import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState, type ReactNode } from "react";

/** How long the pointer rests on a tile before its card opens. */
const OPEN_DELAY_MS = 120;

/** A tile of the pages grid; while a mouse rests on it, a card above shows the page's design. */
export function PageTile({
	picture,
	width,
	height,
	alignment,
	className,
	children,
}: {
	picture: string;
	/** The picture's size in pixels; the card shows it at the width given here. */
	width: number;
	height: number;
	/** Classes that put the card against the tile's left edge, right edge or middle. */
	alignment: string;
	className: string;
	children: ReactNode;
}) {
	const [open, setOpen] = useState(false);
	const timer = useRef<ReturnType<typeof setTimeout> | null>(null);

	useEffect(
		() => () => {
			if (timer.current) clearTimeout(timer.current);
		},
		[],
	);

	function enter(event: React.PointerEvent) {
		if (event.pointerType !== "mouse") return;
		// Start loading the picture now, so it is there when the card opens.
		new Image().src = picture;
		if (timer.current) clearTimeout(timer.current);
		timer.current = setTimeout(() => setOpen(true), OPEN_DELAY_MS);
	}

	function leave() {
		if (timer.current) clearTimeout(timer.current);
		setOpen(false);
	}

	return (
		<div className={className} onPointerEnter={enter} onPointerLeave={leave}>
			{children}
			<div
				aria-hidden="true"
				className={`pointer-events-none absolute bottom-[calc(100%+10px)] z-20 hidden ${alignment} min-[1000px]:[@media(hover:hover)]:block`}
				style={{ width }}
			>
				<AnimatePresence>
					{open && (
						<motion.div
							className="origin-bottom rounded-[18px] bg-card p-1.5 [box-shadow:0_0_0_1px_var(--border),0_24px_48px_-16px_rgba(15,23,42,0.35)]"
							initial={{ opacity: 0, y: 12, scale: 0.95 }}
							animate={{ opacity: 1, y: 0, scale: 1 }}
							exit={{ opacity: 0, y: 6, scale: 0.98, transition: { duration: 0.12 } }}
							transition={{ type: "spring", stiffness: 460, damping: 34, mass: 0.7 }}
						>
							{/* eslint-disable-next-line @next/next/no-img-element */}
							<img
								src={picture}
								width={width}
								height={height}
								alt=""
								decoding="async"
								className="block h-auto w-full rounded-[13px]"
							/>
						</motion.div>
					)}
				</AnimatePresence>
			</div>
		</div>
	);
}
