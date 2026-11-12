export type GitHubRef = {
	owner: string;
	repo: string;
	branch: string;
	/** Path inside the repo to the config directory (default `config`). */
	configDir: string;
};

const DEFAULT_BRANCH = 'main';

/**
 * Accepts:
 * - https://github.com/owner/repo/tree/branch
 * - https://github.com/owner/repo/tree/branch/config
 * - https://github.com/owner/repo
 * - owner/repo
 * - owner/repo@branch
 */
export function parseGitHubUrl(input: string): GitHubRef {
	const trimmed = input.trim().replace(/\/+$/, '');
	if (!trimmed) throw new Error('Enter a GitHub repo URL or owner/repo.');

	const atForm = /^([A-Za-z0-9_.-]+)\/([A-Za-z0-9_.-]+)@([A-Za-z0-9._/-]+)$/.exec(trimmed);
	if (atForm) {
		return {
			owner: atForm[1]!,
			repo: atForm[2]!,
			branch: atForm[3]!,
			configDir: 'config'
		};
	}

	const shortForm = /^([A-Za-z0-9_.-]+)\/([A-Za-z0-9_.-]+)$/.exec(trimmed);
	if (shortForm) {
		return {
			owner: shortForm[1]!,
			repo: shortForm[2]!,
			branch: DEFAULT_BRANCH,
			configDir: 'config'
		};
	}

	let url: URL;
	try {
		url = new URL(trimmed.includes('://') ? trimmed : `https://${trimmed}`);
	} catch {
		throw new Error('Could not parse that as a GitHub URL.');
	}

	if (!/^(www\.)?github\.com$/i.test(url.hostname)) {
		throw new Error('Only github.com URLs are supported.');
	}

	const parts = url.pathname.split('/').filter(Boolean);
	if (parts.length < 2) throw new Error('URL must include owner and repo.');

	const owner = parts[0]!;
	const repo = parts[1]!;
	let branch = DEFAULT_BRANCH;
	let configDir = 'config';

	if (parts[2] === 'tree' || parts[2] === 'blob') {
		if (!parts[3]) throw new Error('Branch name missing from tree/blob URL.');
		branch = decodeURIComponent(parts[3]);
		const rest = parts.slice(4);
		if (rest.length > 0) {
			// …/tree/branch/config or …/tree/branch/config/lily58.keymap
			if (rest[rest.length - 1]?.endsWith('.keymap') || rest[rest.length - 1]?.endsWith('.json')) {
				configDir = rest.slice(0, -1).join('/') || 'config';
			} else {
				configDir = rest.join('/');
			}
		}
	}

	return { owner, repo, branch, configDir };
}

export function rawGitHubUrl(ref: GitHubRef, fileName: string): string {
	const path = [ref.configDir, fileName].filter(Boolean).join('/');
	return `https://raw.githubusercontent.com/${ref.owner}/${ref.repo}/${ref.branch}/${path}`;
}

export function repoTreeUrl(ref: GitHubRef): string {
	return `https://github.com/${ref.owner}/${ref.repo}/tree/${ref.branch}/${ref.configDir}`;
}
