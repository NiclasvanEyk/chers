import { useAdjustableSettings } from "@/lib/settings";
import { ReactNode, useEffect, useId, useRef } from "react";

export function SettingsTrigger(props: any) {
	const { className, ...rest } = props;

	return (
		<button
			className={`h-4 w-4 rounded-full bg-black/50 hover:bg-black/60 dark:bg-white/30 dark:hover:bg-white/20 transition-colors border-2 border-black dark:border-white border-dashed ${
				className ?? ""
			}`}
			title="Open Settings"
			{...rest}
		></button>
	);
}

export function Settings(props: { open: boolean; onClose: () => void }) {
	const settings = useAdjustableSettings();
	const dialog = useRef<HTMLDialogElement | null>(null);

	useEffect(() => {
		if (props.open) {
			dialog.current?.showModal();
		} else {
			dialog.current?.close();
		}
	}, [props.open]);

	return (
		<dialog
			ref={dialog}
			onClose={props.onClose}
			className="absolute backdrop-blur-xl bg-white/70 text-black dark:bg-black/80 dark:text-white rounded shadow p-9 inset-0 z-20 backdrop:bg-transparent"
		>
			<button
				onClick={props.onClose}
				className="text-2xl absolute top-0 right-0 h-12 w-12"
				aria-label="Close Settings"
			>
				&times;
			</button>

			<h2 className="text-2xl mb-7">Settings</h2>

			<div className="space-y-7">
				<Section heading="Gameplay">
					<Toggle
						value={settings.all.highlightLegalMoves}
						onChange={(value) => settings.adjust("highlightLegalMoves", value)}
					>
						Highlight possible moves using dark circles.
					</Toggle>
					<Toggle
						value={settings.all.displayLabels}
						onChange={(value) => settings.adjust("displayLabels", value)}
					>
						Display rank and file labels.
					</Toggle>
					<Toggle
						functional={false}
						value={settings.all.displayCapturedPieces}
						onChange={(value) =>
							settings.adjust("displayCapturedPieces", value)
						}
					>
						Display captured pieces.
					</Toggle>
				</Section>

				<Section heading="Keyboard Navigation">
					<Toggle
						functional={false}
						value={settings.all.onlyLegalTabTargets}
						onChange={(value) => settings.adjust("onlyLegalTabTargets", value)}
					>
						Skip non-selectable cells when tabbing.
					</Toggle>
				</Section>
			</div>
		</dialog>
	);
}

function Section(props: { heading: String; children: ReactNode }) {
	return (
		<section>
			<h3 className="text-xl mb-1">{props.heading}</h3>

			{props.children}
		</section>
	);
}

function Toggle(props: {
	children: ReactNode;
	functional?: boolean;
	value: boolean;
	onChange: (value: boolean) => void;
}) {
	const placeholder = props.functional === false;
	const id = useId();

	return (
		<label
			htmlFor={id}
			className={`flex flex-row items-center select-none ${
				placeholder ? "opacity-50" : ""
			}`}
		>
			<input
				type="checkbox"
				id={id}
				className="mr-3"
				disabled={placeholder}
				checked={props.value}
				onChange={(event) => props.onChange(event.target.checked)}
			/>
			{props.children} {placeholder ? "-- todo" : null}
		</label>
	);
}
