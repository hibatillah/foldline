import { OpenNotch } from "./open-notch";

export function ShelfNotch() {
	return (
		<OpenNotch notch="shelf" activePage={1}>
			<div className="h-[86px] [border:1.5px_dashed_#5a5a68] rounded-[12px] box-border grid grid-cols-[repeat(6,_78px)] [align-items:start] pt-[6px] justify-center">
				<div className="flex flex-col items-center gap-[3px]">
					<div className="w-[40px] h-[40px] flex items-center justify-center">
						<div className="w-[30px] h-[38px] rounded-[3px] bg-[#fff] relative">
							<span className="absolute left-[-3px] bottom-[6px] p-[1px_3px] rounded-[2px] bg-[#e03131] text-[#fff] text-[8px] font-medium">
								PDF
							</span>
						</div>
					</div>
					<span className="w-[70px] text-[10px] leading-[12px] text-[#d6d6de] text-center [display:-webkit-box] [-webkit-line-clamp:2] [-webkit-box-orient:vertical] overflow-hidden">
						Brief v3.pdf
					</span>
				</div>
				<div className="flex flex-col items-center gap-[3px]">
					<div className="w-[40px] h-[40px] flex items-center justify-center">
						<div className="w-[40px] h-[40px] rounded-[6px] [background:linear-gradient(160deg,_#a5d8ff,_#4c6ef5_60%,_#364fc7)]" />
					</div>
					<span className="w-[70px] text-[10px] leading-[12px] text-[#d6d6de] text-center [display:-webkit-box] [-webkit-line-clamp:2] [-webkit-box-orient:vertical] overflow-hidden">
						hero-shot.png
					</span>
				</div>
				<div className="flex flex-col items-center gap-[3px]">
					<div className="w-[40px] h-[40px] flex items-center justify-center">
						<div className="w-[34px] h-[28px] rounded-[4px] bg-[#f8bd3b] relative">
							<span className="absolute left-[15px] top-[4px] w-[4px] h-[18px] [background:repeating-linear-gradient(180deg,_#8a6514_0_2px,_transparent_2px_4px)]" />
						</div>
					</div>
					<span className="w-[70px] text-[10px] leading-[12px] text-[#d6d6de] text-center [display:-webkit-box] [-webkit-line-clamp:2] [-webkit-box-orient:vertical] overflow-hidden">
						assets.zip
					</span>
				</div>
				<div className="flex flex-col items-center gap-[3px]">
					<div className="w-[40px] h-[40px] flex items-center justify-center">
						<div className="w-[30px] h-[38px] rounded-[3px] bg-[#fff] flex flex-col gap-[4px] p-[8px_5px] box-border">
							<span className="h-[1px] bg-[#c4c4cc]" />
							<span className="h-[1px] bg-[#c4c4cc]" />
							<span className="h-[1px] bg-[#c4c4cc]" />
							<span className="h-[1px] w-[60%] bg-[#c4c4cc]" />
						</div>
					</div>
					<span className="w-[70px] text-[10px] leading-[12px] text-[#d6d6de] text-center [display:-webkit-box] [-webkit-line-clamp:2] [-webkit-box-orient:vertical] overflow-hidden">
						notes.txt
					</span>
				</div>
				<div className="flex flex-col items-center gap-[3px]">
					<div className="w-[40px] h-[40px] flex items-center justify-center">
						<div className="w-[30px] h-[38px] rounded-[3px] bg-[#fff] relative overflow-hidden">
							<span className="absolute left-0 top-0 bottom-0 w-[7px] bg-[#2f9e44]" />
							<span className="absolute right-[-2px] bottom-[4px] w-[14px] h-[14px] rounded-[3px] bg-[#107c41] text-[#fff] text-[9px] font-medium flex items-center justify-center">
								X
							</span>
						</div>
					</div>
					<span className="w-[70px] text-[10px] leading-[12px] text-[#d6d6de] text-center [display:-webkit-box] [-webkit-line-clamp:2] [-webkit-box-orient:vertical] overflow-hidden">
						Budget 2026.xlsx
					</span>
				</div>
				<div className="flex flex-col items-center gap-[3px]">
					<div className="w-[40px] h-[40px] flex items-center justify-center">
						<div className="w-[30px] h-[38px] rounded-[3px] bg-[#fff] relative">
							<span className="absolute left-[-3px] bottom-[6px] p-[1px_3px] rounded-[2px] bg-[#3b5bdb] text-[#fff] text-[8px] font-medium">
								DOC
							</span>
						</div>
					</div>
					<span className="w-[70px] text-[10px] leading-[12px] text-[#d6d6de] text-center [display:-webkit-box] [-webkit-line-clamp:2] [-webkit-box-orient:vertical] overflow-hidden">
						Contract.docx
					</span>
				</div>
			</div>
		</OpenNotch>
	);
}
