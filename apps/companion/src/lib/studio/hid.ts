import tables from './hid-tables.json';
import overrides from './hid-overrides.json';

type UsageId = { Id: number; Name: string };
type UsagePage = { Id: number; Name: string; UsageIds?: UsageId[] };

const usagePages = (tables as { UsagePages: UsagePage[] }).UsagePages;
const labelOverrides = overrides as Record<string, Record<string, { short?: string; med?: string; long?: string }>>;

/** Pack HID page + id into the ZMK Studio param encoding. */
export function hidUsageFromPageAndId(page: number, id: number): number {
	return (page << 16) + id;
}

export function hidPageAndIdFromUsage(usage: number): [number, number] {
	return [(usage >> 16) & 0xffff, usage & 0xffff];
}

export function hidUsageLabel(usage: number): string {
	if (!usage) return '—';
	const [page, id] = hidPageAndIdFromUsage(usage);
	const short = labelOverrides[String(page)]?.[String(id)]?.short;
	if (short) return short;
	const pageInfo = usagePages.find((p) => p.Id === page);
	const name = pageInfo?.UsageIds?.find((u) => u.Id === id)?.Name;
	return name ?? `0x${usage.toString(16)}`;
}

export function keyboardUsagePages(): UsagePage[] {
	return usagePages.filter((p) => p.Id === 7 || p.Id === 12);
}

export { usagePages };
