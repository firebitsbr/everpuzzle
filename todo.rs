
Now {
	
}

Bugs {
	Block {
		Swap triggers hang to be chainable?,
		 chain not getting set right,
		chain vertical not getting set right
	}
}

Rendering {
	clipping,
	chaching,
	[x] move to instanced rendering + indices,
}

Helpers {
	[_] iterator through real blocks, block_states, garbage, garbage_states,
	[x] Use IV2 on things that dont need float?,
}

 Option / Nice to have {
	rebind keys,
	ini file for data {
		hot swapped?,
		animation data,
	}
}

General {
	multiple grids,
	multiple players with set controls,
	[x] gamepad input singleplayer,
	
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
	
	Ai {
		gets stuck sometimes,
		recalculate goal when something isnt idle,
	}
}

Block {
	[x] falling stop swap?,
	[x] make L J R Rreverse clears work,
	[x] clear should happen in a row,
	[x] land state,
	
	get_clear_type - from vframe {
		[x] normal,
		Steel,
		etc,
	}
}

Grid {
	lose at top,
	
	Randomization {
	randomization dont let combos happen at spawn,
		gen 6 without matches,
	},
	
	[x] y_offset key_down held till one push is done,
	[x] y offset per pixel,
	[x] clear chain,
	[x] combo count on each frame,
}

Garbage System {
	delay garbage spawn,
	draw new block already when clearing child - till it gets added as real component,
	
	[x] make new spawned blocks from garbage chainable,
	[x] fix garbage clear not ending properly,
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
	[x] other color while clearing,
	[x] check create 3x1 - 4x1 etc,
	}

Game Modes {
	Each level increase get a new grid to manage with the same input,
}
i
4coder {
	add {}, (), [], automatically,
	recognize fn asd<> as a function too,
	#[inline] shouldnt indent next text,
	make brace highlights not highlight indent = 1,
	allow numbers and _ like 100_000,
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
