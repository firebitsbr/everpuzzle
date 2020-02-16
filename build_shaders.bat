@echo off
glslc shaders/grid.vert -o shaders/grid.vert.spv
glslc shaders/grid.frag -o shaders/grid.frag.spv

glslc shaders/sprite.vert -o shaders/sprite.vert.spv
glslc shaders/sprite.frag -o shaders/sprite.frag.spv

glslc shaders/text.vert -o shaders/text.vert.spv
glslc shaders/text.frag -o shaders/text.frag.spv

glslc shaders/quad.vert -o shaders/quad.vert.spv
glslc shaders/quad.frag -o shaders/quad.frag.spv