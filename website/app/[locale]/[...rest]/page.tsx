import { notFound } from "next/navigation";

/** Catches every address no page owns, so the localized not-found page renders inside the locale layout. */
export default function CatchAll() {
	notFound();
}
