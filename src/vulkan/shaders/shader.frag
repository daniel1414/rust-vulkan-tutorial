#version 450

layout (location = 0) in vec3 color;
layout (location = 1) in vec2 texCoord;

layout (location = 0) out vec4 outColor;

//layout(set = 0, binding = 0) uniform sampler2D sampler;

void main() {
    outColor = vec4(texCoord, 0.0, 1.0);
}