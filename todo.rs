
[x] Input	Time: 00:17:50 {
	[x] presses,
	[x] downs,
	[x] frames down,
}

Mouse {
	[x] pressed	Time: 00:25:00,
	[x] down,
	[x] position,
}

Rendering {
	[x] sprite	Time: 00:25:00,
	[x] text	Time: 00:25:00,
	[x] grid,
	Global scale?,
}

Shaders {
	[x] load from file instead,
	hot load shaders,
}

Cursor {
	input {
		[x] simple,
		increase speed when frame time is reached,
	},
	
	[x] rendering	Time: 00:25:00,
	animation,
	
	actions {
		[x] test,
		[x] swap,
	},
}

Grid {
	[x] basics	Time: 00:50:00,
	interaction between components - has to live in grid.rs it seems,
}

Block {
	lagless fall	Time: 00:25:00,
}

General {
	iterator that does reverse x and y for index automatically,
}

4coder {
	turn of pomodoro command,
	color pomodoro break times all the time,
	color long break differently,
	
	Snippets {
		fnx marker - 1,
		match bad breaks,
		dont return \n with snippets,
	}
	
	pomodoro if paste before time limit reached - start from 0 go til i.e. 25 - stop point,
	fix . only work on empty new line on left,
	
	swap between last window,
	' highlight only if non identifier follows,
}
