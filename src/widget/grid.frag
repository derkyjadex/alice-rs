#version 140

uniform vec3 grid_colour;

in vec2 grid_coords;
in vec2 grid_step;

out vec4 color;

void main() {
    vec2 grid = step(grid_step, fract(grid_coords));
    float alpha = 1.0 - min(grid.x, grid.y);

    color = vec4(grid_colour, alpha);
}
