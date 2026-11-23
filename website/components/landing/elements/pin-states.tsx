import { useTranslations } from "next-intl";

export function PinStates() {
	const t = useTranslations();

	return (
		<div className="flex gap-[40px] items-center">
			<div className="flex flex-col items-center gap-[12px]">
				<svg
					aria-hidden="true"
					width="38"
					height="38"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#a1a1aa"
					strokeWidth="1.6"
					strokeLinecap="round"
					strokeLinejoin="round"
				>
					<path d="M12 17v5" />
					<path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z" />
				</svg>
				<span className="text-[12px] text-[#71717a]">{t("word.unpinned")}</span>
			</div>
			<div className="flex flex-col items-center gap-[12px]">
				<svg
					aria-hidden="true"
					width="38"
					height="38"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#18181b"
					strokeWidth="1.6"
					strokeLinecap="round"
					strokeLinejoin="round"
				>
					<path d="M12 17v5" />
					<path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z" />
				</svg>
				<span className="text-[12px] text-[#71717a]">{t("word.pinned")}</span>
			</div>
		</div>
	);
}
