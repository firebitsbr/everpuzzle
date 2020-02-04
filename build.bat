@echo off
glslc shaders/grid.vert -o shaders/grid.vert.spv
glslc shaders/grid.vert -o target/debug/shaders/grid.vert.spv

glslc shaders/grid.frag -o shaders/grid.frag.spv
glslc shaders/grid.vert -o target/debug/shaders/grid.frag.spv


glslc shaders/sprite.vert -o shaders/sprite.vert.spv
glslc shaders/sprite.vert -o target/debug/shaders/sprite.vert.spv

glslc shaders/sprite.frag -o shaders/sprite.frag.spv
glslc shaders/sprite.vert -o target/debug/shaders/sprite.frag.spv