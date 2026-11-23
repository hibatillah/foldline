import { useTranslations } from "next-intl";

export function StatusTiles() {
	const t = useTranslations();

	return (
		<div className="w-full max-w-[420px] grid grid-cols-[repeat(2,_minmax(0,_1fr))] gap-[10px]">
			<div className="h-[56px] rounded-[12px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] flex items-center gap-[12px] p-[0_16px] text-[14px] text-[#18181b]">
				<svg
					aria-hidden="true"
					width="18"
					height="18"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#18181b"
					strokeWidth="2"
					strokeLinejoin="round"
				>
					<path d="M12 4 4 12h4v5h8v-5h4z" />
					<path d="M8 20h8" />
				</svg>
				<span>Caps Lock</span>
			</div>
			<div className="h-[56px] rounded-[12px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] flex items-center gap-[12px] p-[0_16px] text-[14px] text-[#18181b]">
				<svg
					aria-hidden="true"
					width="18"
					height="18"
					viewBox="0 0 24 24"
					fill="none"
					stroke="#18181b"
					strokeWidth="2"
				>
					<path d="M11 5 6 9H3v6h3l5 4z" />
					<path d="m16 9 5 6M21 9l-5 6" />
				</svg>
				<span>{t("word.muted")}</span>
			</div>
			<div className="h-[56px] rounded-[12px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] flex items-center gap-[12px] p-[0_16px] text-[14px] text-[#18181b]">
				<span className="w-[10px] h-[10px] rounded-full bg-[#2f9e44] m-[0_4px]" />
				<span>{t("word.camera")}</span>
			</div>
			<div className="h-[56px] rounded-[12px] bg-[#ffffff] [box-shadow:0_0_0_1px_#e4e4e7] flex items-center gap-[12px] p-[0_16px] text-[14px] text-[#18181b]">
				<span className="w-[10px] h-[10px] rounded-full bg-[#f08c00] m-[0_4px]" />
				<span>{t("word.microphone")}</span>
			</div>
		</div>
	);
}
