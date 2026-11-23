import { OpenNotch } from "./open-notch";

export function ClipboardNotch() {
	return (
		<OpenNotch notch="clipboard" activePage={8}>
			<div className="h-[179px] flex gap-[12px]">
				<div className="w-[214px] flex-none flex flex-col gap-[3px]">
					<div className="h-[23px] rounded-[8px] bg-[#1a1a20] [border:1px_solid_#3a3a44] box-border flex items-center gap-[7px] p-[0_8px] text-[11px] text-[#e6e6eb]">
						<svg
							aria-hidden="true"
							width="14"
							height="14"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#8a8a96"
							strokeWidth="1.8"
							strokeLinecap="round"
							strokeLinejoin="round"
							className="flex-none"
						>
							<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
							<path d="M14 2v6h6M8 13h8M8 17h6" />
						</svg>
						<span className="whitespace-nowrap overflow-hidden text-ellipsis">
							Meeting moved to 15:30. Bring the release checklist and the screenshots for the new
							site, and we will go through the open pull requests before tagging 0.6.0.
						</span>
					</div>
					<div className="h-[23px] rounded-[8px] bg-[#121216] [border:1px_solid_#1e1e24] box-border flex items-center gap-[7px] p-[0_8px] text-[11px] text-[#e6e6eb]">
						<svg
							aria-hidden="true"
							width="14"
							height="14"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#8a8a96"
							strokeWidth="1.8"
							strokeLinecap="round"
							strokeLinejoin="round"
							className="flex-none"
						>
							<path d="M10 13a5 5 0 0 0 7.5.5l3-3a5 5 0 0 0-7-7l-1.7 1.7M14 11a5 5 0 0 0-7.5-.5l-3 3a5 5 0 0 0 7 7l1.7-1.7" />
						</svg>
						<span className="whitespace-nowrap overflow-hidden text-ellipsis">
							fold.hibatillah.com/releases
						</span>
					</div>
					<div className="h-[23px] rounded-[8px] bg-[#121216] [border:1px_solid_#1e1e24] box-border flex items-center gap-[7px] p-[0_8px] text-[11px] text-[#e6e6eb]">
						<span className="w-[14px] h-[14px] rounded-[4px] bg-[#2563eb] flex-none" />
						<span className="whitespace-nowrap overflow-hidden text-ellipsis">#2563eb</span>
					</div>
					<div className="h-[23px] rounded-[8px] bg-[#121216] [border:1px_solid_#1e1e24] box-border flex items-center gap-[7px] p-[0_8px] text-[#e6e6eb] [font-family:Consolas,_monospace] text-[10.5px]">
						<svg
							aria-hidden="true"
							width="14"
							height="14"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#8a8a96"
							strokeWidth="1.8"
							strokeLinecap="round"
							strokeLinejoin="round"
							className="flex-none"
						>
							<path d="m16 18 6-6-6-6M8 6l-6 6 6 6" />
						</svg>
						<span className="whitespace-nowrap overflow-hidden text-ellipsis">
							cargo build --release
						</span>
					</div>
					<div className="h-[23px] rounded-[8px] bg-[#121216] [border:1px_solid_#1e1e24] box-border flex items-center gap-[7px] p-[0_8px] text-[11px] text-[#e6e6eb]">
						<svg
							aria-hidden="true"
							width="14"
							height="14"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#8a8a96"
							strokeWidth="1.8"
							strokeLinecap="round"
							strokeLinejoin="round"
							className="flex-none"
						>
							<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
							<path d="M14 2v6h6M8 13h8M8 17h6" />
						</svg>
						<span className="whitespace-nowrap overflow-hidden text-ellipsis">Pekanbaru, Riau</span>
					</div>
					<div className="h-[23px] rounded-[8px] bg-[#121216] [border:1px_solid_#1e1e24] box-border flex items-center gap-[7px] p-[0_8px] text-[11px] text-[#e6e6eb]">
						<span className="w-[15px] h-[15px] rounded-[4px] [background:linear-gradient(160deg,_#a5d8ff,_#4c6ef5)] flex-none" />
						<span className="whitespace-nowrap overflow-hidden text-ellipsis">Image (360x70)</span>
					</div>
					<div className="h-[23px] rounded-[8px] bg-[#121216] [border:1px_solid_#1e1e24] box-border flex items-center gap-[7px] p-[0_8px] text-[11px] text-[#e6e6eb]">
						<svg
							aria-hidden="true"
							width="14"
							height="14"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#8a8a96"
							strokeWidth="1.8"
							strokeLinecap="round"
							strokeLinejoin="round"
							className="flex-none"
						>
							<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
							<path d="M14 2v6h6M8 13h8M8 17h6" />
						</svg>
						<span className="whitespace-nowrap overflow-hidden text-ellipsis">
							Thanks, see you tomorrow
						</span>
					</div>
				</div>
				<div className="flex-1 flex flex-col gap-[8px]">
					<div className="flex-1 rounded-[8px] bg-[#0b0b0e] p-[8px_10px] text-[12px] leading-[17px] text-[#e6e6eb]">
						Meeting moved to 15:30. Bring the release checklist and the screenshots for the new
						site, and we will go through the open pull requests before tagging 0.6.0.
					</div>
					<div className="h-[26px] flex items-center gap-[6px]">
						<div className="flex flex-col text-[10px] leading-[12px] text-[#6e6e7a] mr-[auto]">
							<span>Text · 157 characters</span>
							<span>just now</span>
						</div>
						<span className="w-[26px] h-[26px] rounded-[8px] bg-[#121216] [border:1px_solid_#2c2c34] box-border flex items-center justify-center">
							<svg
								aria-hidden="true"
								width="12"
								height="12"
								viewBox="0 0 24 24"
								fill="none"
								stroke="#ff6b6b"
								strokeWidth="2"
								strokeLinecap="round"
							>
								<path d="M3 6h18M8 6V4h8v2M6 6l1 14h10l1-14" />
							</svg>
						</span>
						<span className="w-[26px] h-[26px] rounded-[8px] bg-[#121216] [border:1px_solid_#2c2c34] box-border flex items-center justify-center">
							<svg
								aria-hidden="true"
								width="12"
								height="12"
								viewBox="0 0 24 24"
								fill="none"
								stroke="#9a9aa6"
								strokeWidth="2"
							>
								<rect x="8" y="8" width="13" height="13" rx="2" />
								<path d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3" />
							</svg>
						</span>
						<span className="w-[76px] h-[26px] rounded-[8px] bg-[#fff] text-[#000] text-[12px] font-medium flex items-center justify-center">
							Paste
						</span>
					</div>
				</div>
			</div>
		</OpenNotch>
	);
}
