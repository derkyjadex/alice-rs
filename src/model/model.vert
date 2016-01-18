#version 140

uniform vec2 viewport_size;
uniform vec2 translate;
uniform float scale;

in vec2 position;
in vec3 param;

out vec3 p;

void main() {
    vec2 pos = translate + scale * position;
    gl_Position = vec4(vec2(2) * pos / viewport_size - vec2(1), 0, 1);
    p = param;
}