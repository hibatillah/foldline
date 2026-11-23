import { Banner } from "./banner";

export function DownloadBanner() {
	return (
		<Banner notch="download">
			<div className="flex gap-[12px] items-start">
				<div className="flex-none w-[30px] h-[30px] flex items-center justify-center">
					<div className="w-[24px] h-[30px] rounded-[3px] bg-[#fff] relative">
						<span className="absolute left-[-3px] bottom-[6px] p-[1px_3px] rounded-[2px] bg-[#e03131] text-[#fff] text-[8px] font-medium">
							PDF
						</span>
					</div>
				</div>
				<div className="flex flex-col gap-[2px] min-w-0 flex-1">
					<div className="text-[14px] font-medium whitespace-nowrap overflow-hidden text-ellipsis">
						Annual report 2026.pdf
					</div>
					<div className="text-[13px] text-[#adadb8]">Downloaded · 17.8 MB</div>
					<div className="self-end mt-[10px] flex gap-[8px]">
						<span className="h-[28px] p-[0_14px] rounded-[14px] bg-[rgba(255,255,255,0.14)] text-[#fff] text-[13px] font-medium flex items-center">
							Show in folder
						</span>
						<span className="h-[28px] p-[0_14px] rounded-[14px] bg-[#fff] text-[#000] text-[13px] font-medium flex items-center">
							Open
						</span>
					</div>
				</div>
			</div>
		</Banner>
	);
}
