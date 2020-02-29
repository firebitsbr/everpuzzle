@echo off
glslc shaders/quad.vert -o shaders/quad.vert.spv
glslc shaders/quad.frag -o shaders/quad.frag.spv

glslc shaders/line.vert -o shaders/line.vert.spv
glslc shaders/line.frag -o shaders/line.frag.spv