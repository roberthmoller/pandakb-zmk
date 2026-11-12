const DEFAULT_WEST_ROOT =
	'/Users/robert/Documents/Codex/2026-11-05/referenced-chatgpt-conversation-this-is-an/work/pandakb-lily';

export type FirmwarePanel = 'live' | 'build';

class FirmwareState {
	panel = $state<FirmwarePanel>('live');
	westRoot = $state(
		(typeof localStorage !== 'undefined' && localStorage.getItem('zmkwiz.westRoot')) ||
			DEFAULT_WEST_ROOT
	);
	buildLog = $state('');
	buildStatus = $state<'idle' | 'running' | 'ok' | 'error'>('idle');
	artifacts = $state<{ left?: string; right?: string }>({});
	flashStatus = $state<string | null>(null);

	setPanel(panel: FirmwarePanel) {
		this.panel = panel;
	}

	persistWestRoot() {
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem('zmkwiz.westRoot', this.westRoot);
		}
	}
}

export const firmwareState = new FirmwareState();
export { DEFAULT_WEST_ROOT };
