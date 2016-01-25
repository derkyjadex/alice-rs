#version 140

uniform vec2 viewport_size;
uniform vec2 location;
uniform vec2 size;
uniform float border_width;

in vec2 position;

out vec2 border_coords;
out vec2 border_step;

void main() {
    vec2 _location = floor(location);
    vec2 _size = floor(size);
    float _border_width = floor(border_width);

    border_coords = position - 0.5;
    border_step = 0.5 - _border_width / _size;

    vec2 pos = _location + _size * position;
    gl_Position = vec4(vec2(2) * pos / viewport_size - vec2(1), 0, 1);
}
