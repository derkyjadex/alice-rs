#version 140

uniform vec3 colour;

in vec3 p;

out vec4 color;

void main() {
    float s = p.x * p.x - p.y;
    float a = step(0.0, p.z * s);

    color = vec4(colour, a);
}