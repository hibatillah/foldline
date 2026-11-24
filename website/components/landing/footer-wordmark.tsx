"use client";

import { motion, type Variants } from "motion/react";

/** The letters with ids of their own, since a letter repeats in the name. */
const LETTERS = [
	{ id: "f", letter: "F" },
	{ id: "o", letter: "o" },
	{ id: "l-first", letter: "l" },
	{ id: "d", letter: "d" },
	{ id: "l-second", letter: "l" },
	{ id: "i", letter: "i" },
	{ id: "n", letter: "n" },
	{ id: "e", letter: "e" },
];

const WORD: Variants = {
	hidden: {},
	shown: { transition: { staggerChildren: 0.03 } },
};

const LETTER: Variants = {
	hidden: { y: "55%", opacity: 0 },
	shown: { y: 0, opacity: 1, transition: { duration: 0.5, ease: [0.16, 1, 0.3, 1] } },
};

/** The large name behind the footer; its letters rise one after another when the footer scrolls into view. */
export function FooterWordmark() {
	return (
		<motion.div
			aria-hidden="true"
			className="pointer-events-none absolute bottom-[-0.2em] left-[50%] flex select-none text-[clamp(120px,21vw,300px)] leading-[0.8] font-semibold tracking-[-0.06em] whitespace-nowrap text-[oklch(0.935_0.012_250)] [transform:translateX(-50%)] max-[640px]:text-[25vw]"
			variants={WORD}
			initial="hidden"
			whileInView="shown"
			viewport={{ once: true, amount: 0.3 }}
		>
			{LETTERS.map(({ id, letter }) => (
				<motion.span key={id} className="inline-block" variants={LETTER}>
					{letter}
				</motion.span>
			))}
		</motion.div>
	);
}
