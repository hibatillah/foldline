import { useTranslations } from "next-intl";

export function BatteryRings() {
	const t = useTranslations();

	return (
		<div className="flex gap-[24px] items-center">
			<div className="flex flex-col items-center gap-[8px]">
				<svg aria-hidden="true" width="46" height="46" viewBox="0 0 16 16">
					<circle cx="8" cy="8" r="6" fill="none" stroke="#e4e4e7" strokeWidth="1.6" />
					<circle
						cx="8"
						cy="8"
						r="6"
						fill="none"
						stroke="#2f9e44"
						strokeWidth="1.6"
						strokeLinecap="round"
						strokeDasharray="26.4 37.7"
						transform="rotate(-90 8 8)"
					/>
				</svg>
				<span className="flex flex-col items-center gap-[2px]">
					<span className="text-[13px] font-medium text-[#18181b]">70%</span>
					<span className="text-[11px] text-[#71717a]">{t("word.idle")}</span>
				</span>
			</div>
			<div className="flex flex-col items-center gap-[8px]">
				<svg aria-hidden="true" width="46" height="46" viewBox="0 0 16 16">
					<circle cx="8" cy="8" r="6" fill="none" stroke="#e4e4e7" strokeWidth="1.6" />
					<circle
						cx="8"
						cy="8"
						r="6"
						fill="none"
						stroke="#2b8a3e"
						strokeWidth="1.6"
						strokeLinecap="round"
						strokeDasharray="21.1 37.7"
						transform="rotate(-90 8 8)"
					/>
					<path d="M8.6 4.6 5.8 8.6h2l-.5 2.8 2.9-4h-2z" fill="#2b8a3e" />
				</svg>
				<span className="flex flex-col items-center gap-[2px]">
					<span className="text-[13px] font-medium text-[#18181b]">56%</span>
					<span className="text-[11px] text-[#71717a]">{t("word.charging")}</span>
				</span>
			</div>
			<div className="flex flex-col items-center gap-[8px]">
				<svg aria-hidden="true" width="46" height="46" viewBox="0 0 16 16">
					<circle cx="8" cy="8" r="6" fill="none" stroke="#e4e4e7" strokeWidth="1.6" />
					<circle
						cx="8"
						cy="8"
						r="6"
						fill="none"
						stroke="#e03131"
						strokeWidth="1.6"
						strokeLinecap="round"
						strokeDasharray="6.8 37.7"
						transform="rotate(-90 8 8)"
					/>
				</svg>
				<span className="flex flex-col items-center gap-[2px]">
					<span className="text-[13px] font-medium text-[#e03131]">18%</span>
					<span className="text-[11px] text-[#71717a]">{t("word.low")}</span>
				</span>
			</div>
		</div>
	);
}
