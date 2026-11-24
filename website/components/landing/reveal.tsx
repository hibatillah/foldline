"use client";

import { MotionConfig, motion, type Variants } from "motion/react";
import type { ReactNode } from "react";

const EASE_OUT = [0.16, 1, 0.3, 1] as const;

/** Enter animations of the landing page; reduced motion keeps the fade and drops the movement. */
export function MotionRoot({ children }: { children: ReactNode }) {
	return <MotionConfig reducedMotion="user">{children}</MotionConfig>;
}

/** Fades a block in and lifts it a little the first time it scrolls into view. */
export function Reveal({
	children,
	delay = 0,
	distance = 20,
	className,
}: {
	children: ReactNode;
	delay?: number;
	distance?: number;
	className?: string;
}) {
	return (
		<motion.div
			className={className}
			initial={{ opacity: 0, y: distance }}
			whileInView={{ opacity: 1, y: 0 }}
			viewport={{ once: true, margin: "0px 0px -80px 0px" }}
			transition={{ duration: 0.6, delay, ease: EASE_OUT }}
		>
			{children}
		</motion.div>
	);
}

const GROUP: Variants = {
	hidden: {},
	shown: { transition: { staggerChildren: 0.06 } },
};

const ITEM: Variants = {
	hidden: { opacity: 0, y: 16 },
	shown: { opacity: 1, y: 0, transition: { duration: 0.5, ease: EASE_OUT } },
};

/** A group whose `RevealItem` children enter one after the other when the group scrolls into view. */
export function RevealGroup({ children, className }: { children: ReactNode; className?: string }) {
	return (
		<motion.div
			className={className}
			variants={GROUP}
			initial="hidden"
			whileInView="shown"
			viewport={{ once: true, margin: "0px 0px -80px 0px" }}
		>
			{children}
		</motion.div>
	);
}

export function RevealItem({ children, className }: { children: ReactNode; className?: string }) {
	return (
		<motion.div className={className} variants={ITEM}>
			{children}
		</motion.div>
	);
}
