import Image from "next/image";

import { OpenNotch, MediaStatus } from "./open-notch";

/** A lyric line: one centered line that never wraps. */
const LYRIC = "truncate text-center";

/** The album cover shown as the track artwork. */
function Artwork() {
	return (
		<Image
			src="/artwork/stuck-with-u.jpg"
			alt=""
			width={86}
			height={86}
			unoptimized
			className="block rounded-[12px]"
		/>
	);
}

/** The streaming app mark that sits on the artwork corner, as in the app. */
function SourceBadge() {
	return (
		<span className="absolute right-[-1px] bottom-[-1px] flex size-6 items-center justify-center rounded-[7px] bg-[#0c0c0c] [box-shadow:0_0_0_3px_#000]">
			<Image
				src="/artwork/spotify.png"
				alt=""
				width={20}
				height={20}
				unoptimized
				className="block size-5"
			/>
		</span>
	);
}

export function MediaNotch() {
	return (
		<OpenNotch
			notch="media"
			activePage={0}
			background="bg-[linear-gradient(180deg,#000_0%,#0e0e0f_35%,#2d2d2f_100%)]"
			status={<MediaStatus />}
		>
			<div className="h-[86px] flex gap-[20px] items-center">
				<div className="relative flex-none w-[86px] h-[86px]">
					<Artwork />
					<SourceBadge />
				</div>
				<div className="flex-none w-[180px] flex flex-col gap-[2px]">
					<div className="text-[14px] font-medium leading-[1.25]">
						Stuck with U (with Justin Bieber)
					</div>
					<div className="text-[12px] text-[#9a9aa6]">Ariana Grande</div>
					<div className="flex gap-[14px] items-center mt-[8px]">
						<svg aria-hidden="true" fill="#fff" width="16" height="16" viewBox="0 0 24 24">
							<path d="M6 5h2v14H6zM20 5v14L9 12z" />
						</svg>
						<svg aria-hidden="true" fill="#fff" width="16" height="16" viewBox="0 0 24 24">
							<path d="M7 4.5v15L19 12z" />
						</svg>
						<svg aria-hidden="true" fill="#fff" width="16" height="16" viewBox="0 0 24 24">
							<path d="M16 5h2v14h-2zM4 5v14l11-7z" />
						</svg>
						<svg
							aria-hidden="true"
							width="18"
							height="18"
							viewBox="0 0 24 24"
							fill="none"
							stroke="#30d158"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
						>
							<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
							<path d="M8 8h8M8 12h5" />
						</svg>
					</div>
				</div>
				<div className="flex-1 min-w-0 flex flex-col gap-[10px]">
					<span className={`${LYRIC} text-[13px] text-[rgba(255,255,255,0.2)]`}>
						Count the lights along the road
					</span>
					<span className={`${LYRIC} text-[17px] font-medium`}>We were never far from home</span>
					<span className={`${LYRIC} text-[13px] text-[rgba(255,255,255,0.2)]`}>
						Leave the window open tonight
					</span>
				</div>
			</div>
		</OpenNotch>
	);
}
