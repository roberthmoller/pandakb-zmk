export type PhysicalKey = {
	index: number;
	row: number;
	col: number;
	x: number;
	y: number;
	w: number;
	h: number;
	r: number;
	rx: number;
	ry: number;
};

export type Binding = {
	raw: string;
	behavior: string;
	params: string[];
	label: string;
	gloss: string;
};

export type Layer = {
	id: string;
	name: string;
	bindings: Binding[];
};

export type BoardDisplay = {
	width: number;
	height: number;
	kind: 'oled';
};

export type BoardDefinition = {
	id: string;
	name: string;
	vendor: string;
	keys: PhysicalKey[];
	layers: Layer[];
	display: BoardDisplay;
	sources: { title: string; url: string }[];
};
