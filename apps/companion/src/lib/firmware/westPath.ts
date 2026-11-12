import { resolve } from 'node:path';

/** Only allow absolute paths under the user's home for safety. */
export function assertSafeWestRoot(root: string): string {
	const resolved = resolve(root);
	const home = process.env.HOME;
	if (!home || !resolved.startsWith(home)) {
		throw new Error('WEST_ROOT must be an absolute path under your home directory');
	}
	return resolved;
}
