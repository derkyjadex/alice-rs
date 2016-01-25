#version 140

uniform vec2 viewport_size;
uniform vec2 location;
uniform vec2 size;
uniform vec2 grid_size;
uniform vec2 grid_offset;
const float _grid_width = 1.0;

in vec2 position;

out vec2 grid_coords;
out vec2 grid_step;

void main() {
    vec2 _location = floor(location);
    vec2 _size = floor(size);
    vec2 _grid_size = floor(grid_size);
    vec2 _grid_offset = floor(grid_offset);

    grid_coords = (_size * position - _grid_offset) / _grid_size;
    grid_step = _grid_width / _grid_size;

    vec2 pos = _location + _size * position;
    gl_Position = vec4(vec2(2) * pos / viewport_size - vec2(1), 0, 1);
}
