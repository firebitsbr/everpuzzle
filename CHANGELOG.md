# Everpuzzle 0.1.2:

* Added basic Gamepad button support for a single player
* TAB toggles debug info
* Garbage children have different hframes dependant on their position
* YOffset for components and cursor based on time
* Component Chainable added - adds up chain counting for block that swap with it
* Block Land state with animation
* Combo / Chain highlight
* SPACE pushes components upwards now, pressing push once smoothly raises everything once 

* Fix: Shape clearing "L" "J" etc.
* Fix: Garbage clear was stuck at the end
* Fix: Disallow cursor.swap_blocks if left / right above state is hang

* Improvement: Removed BoundIterator
* Improvement: Removed BoundIndex, switched to using raw usizes, i32 when going outside of bounds
* Improvement: Enum component scripting simplified, not many generalized calls anymore
* Improvement: Combo / Chain highlight with different hframe
* Improvement: Textures adjusted
* Improvement: New Garbage texture
* Improvement: BlockState and GarbageState generated via macro
* Improvement: Personal todo file online

* Experiments: I worked on making scripting easier for 3 days, but never got the result I wanted. In the end it led me to ECS as the only viable option. As the enum approach works nicely so far I want to stick with it for the long run. I made some changes to the workflow, not relying to much on generalized enum calls, but rather on using / initialising enums at the place where I need them. Either way, I think it's always worth exploring other design patterns, it's just that rust makes scripting a bit more tedious / harder than it should be imo.