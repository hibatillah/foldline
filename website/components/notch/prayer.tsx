import { OpenNotch } from "./open-notch";

export function PrayerNotch() {
	return (
		<OpenNotch notch="prayer" activePage={5}>
			<div className="h-[86px] flex items-center gap-[16px]">
				<div className="w-[124px] flex-none flex flex-col gap-[1px]">
					<span className="text-[11px] text-[#9a9aa6]">Next</span>
					<span className="text-[26px] font-medium leading-[30px]">Asr</span>
					<span className="text-[12px] text-[#63e6be]">in 24 min</span>
					<span className="text-[10px] text-[#6e6e7a] mt-[2px]">Pekanbaru</span>
				</div>
				<svg
					aria-hidden="true"
					viewBox="0 0 351 86"
					width="351"
					height="86"
					className="block [font-family:Geist,_sans-serif]"
				>
					<defs>
						<linearGradient id="fl-arc" x1="0" x2="0" y1="0" y2="1">
							<stop offset="0" stopColor="#63e6be" stopOpacity=".28" />
							<stop offset="1" stopColor="#63e6be" stopOpacity="0" />
						</linearGradient>
						<clipPath id="fl-past">
							<rect x="0" y="0" width="140" height="86" />
						</clipPath>
						<clipPath id="fl-next">
							<rect x="140" y="0" width="211" height="86" />
						</clipPath>
					</defs>
					<path
						clipPath="url(#fl-past)"
						d="M35 50 C47 44 82 18 105 14 C129 10 152 19 175 23 C199 28 222 36 246 40 C269 45 304 48 316 50 L316 50 L35 50 Z"
						fill="url(#fl-arc)"
					/>
					<path
						clipPath="url(#fl-next)"
						d="M35 50 C47 44 82 18 105 14 C129 10 152 19 175 23 C199 28 222 36 246 40 C269 45 304 48 316 50"
						fill="none"
						stroke="#3a3a44"
						strokeWidth="2"
						strokeLinecap="round"
					/>
					<path
						clipPath="url(#fl-past)"
						d="M35 50 C47 44 82 18 105 14 C129 10 152 19 175 23 C199 28 222 36 246 40 C269 45 304 48 316 50"
						fill="none"
						stroke="#63e6be"
						strokeWidth="2"
						strokeLinecap="round"
					/>
					<circle cx="35" cy="50" r="3.5" fill="#63e6be" />
					<text x="35" y="68" textAnchor="middle" fontSize="10" fill="#8a8a96">
						Fajr
					</text>
					<text x="35" y="82" textAnchor="middle" fontSize="11" fill="#c4c4cc">
						04:36
					</text>
					<circle cx="105" cy="14" r="3.5" fill="#63e6be" />
					<text x="105" y="68" textAnchor="middle" fontSize="10" fill="#8a8a96">
						Dhuhr
					</text>
					<text x="105" y="82" textAnchor="middle" fontSize="11" fill="#c4c4cc">
						12:00
					</text>
					<circle cx="175" cy="23" r="8" fill="#63e6be" fillOpacity=".18" />
					<circle cx="175" cy="23" r="4.5" fill="#63e6be" />
					<text x="175" y="68" textAnchor="middle" fontSize="10" fill="#63e6be">
						Asr
					</text>
					<text x="175" y="82" textAnchor="middle" fontSize="11" fill="#fff">
						15:19
					</text>
					<circle cx="246" cy="40" r="3.5" fill="#000" stroke="#5a5a66" strokeWidth="1.5" />
					<text x="246" y="68" textAnchor="middle" fontSize="10" fill="#8a8a96">
						Maghrib
					</text>
					<text x="246" y="82" textAnchor="middle" fontSize="11" fill="#c4c4cc">
						18:02
					</text>
					<circle cx="316" cy="50" r="3.5" fill="#000" stroke="#5a5a66" strokeWidth="1.5" />
					<text x="316" y="68" textAnchor="middle" fontSize="10" fill="#8a8a96">
						Isha
					</text>
					<text x="316" y="82" textAnchor="middle" fontSize="11" fill="#c4c4cc">
						19:13
					</text>
					<circle cx="140" cy="16" r="10" fill="#ffd43b" fillOpacity=".25" />
					<circle cx="140" cy="16" r="5.5" fill="#ffd43b" stroke="#000" strokeWidth="1.5" />
				</svg>
			</div>
		</OpenNotch>
	);
}
