import { useState } from "react";
import { Board } from "./Board";
import { ChersSettingsProvider } from "@/lib/settings";
import { Settings, SettingsTrigger } from "./Settings";
import { useChers } from "@/lib/ui/state";

export default function Chers() {
	const [settingsOpen, setSettingsOpen] = useState(false);
	const [state, dispatch] = useChers();

	return (
		<ChersSettingsProvider>
			<SettingsTrigger
				onClick={() => setSettingsOpen(true)}
				className="fixed top-3 left-3 z-10"
				color={state.game.player ?? "white"}
			/>
			<Settings open={settingsOpen} onClose={() => setSettingsOpen(false)} />

			<div className="relative touch-manipulation">
				<Board state={state} dispatch={dispatch} />
			</div>
		</ChersSettingsProvider>
	);
}
