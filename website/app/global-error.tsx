"use client";

/** The last fallback, for an error in the root layout itself: plain English, no site styles, no translations. */
export default function GlobalError({ retry }: { error: unknown; retry: () => void }) {
	return (
		<html lang="en">
			<body className="m-0 min-h-[100vh] flex flex-col items-center justify-center gap-[16px] p-[24px] [font-family:system-ui,_sans-serif] text-[#0f172a] bg-[#fff] text-center">
				<h1 className="m-0 text-[32px] font-medium">Something went wrong</h1>
				<p className="m-0 text-[#64748b]">Foldline hit an unexpected error.</p>
				<div className="flex gap-[12px]">
					<button
						type="button"
						onClick={() => retry()}
						className="p-[10px_20px] rounded-[8px] [border:0] bg-[#2563eb] text-[#fff] text-[15px] cursor-pointer"
					>
						Try again
					</button>
					{/* A full page load, because the router itself may be what broke. */}
					<button
						type="button"
						onClick={() => window.location.assign("/")}
						className="p-[10px_20px] rounded-[8px] [border:1px_solid_#cbd5e1] bg-[#fff] text-[#0f172a] text-[15px] cursor-pointer"
					>
						Back to Foldline
					</button>
				</div>
			</body>
		</html>
	);
}
