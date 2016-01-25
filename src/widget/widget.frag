#version 140

uniform vec4 fill_colour;
uniform vec3 border_colour;

in vec2 border_coords;
in vec2 border_step;

out vec4 color;

void main() {
    vec2 border = step(border_step, abs(border_coords));
    color = mix(fill_colour, vec4(border_colour, 1.0), max(border.x, border.y));
}
