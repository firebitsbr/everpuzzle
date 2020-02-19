
Rendering {
	clipping,
	chaching,
	
	[x] move to instanced rendering + indices,
	[x] text rendering back again,
}

General {
	[_] Use IV2 on things that dont need float?,
	
	multiple grids,
	multiple players with set controls,
	rebind keys,
	gamepad input,
	
	extend grid height to 24 {
		lookup each frame if a block exists in the top space - disable polling in the range if nothing,
		or have 2 vectors,
	}
	
	test game on lower hz,
}

Cursor {
	[x] frame based smooth movement,
	[x] dt movement?,
	[x] smooth animation,
}

Block {
	falling stop swap?,
	[x] clear should happen in a row,
	land state,
	
	get_clear_type - from vframe {
		normal,
		Steel,
		etc,
	}
}

Grid {
	[!] clear chain,
	combo count on each frame,
	y offset per pixel,
	[_] have block_state(i) in bounds dependant on the type -> if option return option else ref,
}

Garbage System {
	delay garbage spawn,
	
	[x] clear should be checked from other garbage too,
	
	[x] 2d {
		[x] clear check all highest/lowest children,
		[x] only clear the lowest blocks - not all!,
	}
	
	[x] 1d {
		[x] clear flood check - left - right - up - down,
	}
	
	[x] clear animation {
		[x] based on child count?,
		[x] GarbageChild have counter and end_counter - since each is unique,
		[x] BlockState::WasGarbage to halt blocks behaviour when still clearing other child but already converted,
	}
	
	[x] other garbage falling on top of other garbage should also get smooth hang,
	[_] other color while clearing,
	[x] check create 3x1 - 4x1 etc,
	}

4coder {
	add {}, (), [], automatically,
	recognize fn asd<> as a function too,
	#[inline] shouldnt indent next text,
}

marketing {
	[x] Video explaining everpuzzle and rebuilt,
	
	Support {
		Play / Feedback / Follow Development,
		Add Github Issues,
		Pull Requests,
		Donations - streams,
	},
	
	[x] Readme {
		[x] gifs,
		[x] goal,
		how to help,
	},
}
