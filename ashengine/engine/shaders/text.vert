#version 450

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec2 inTexCoord;
layout(location = 2) in vec4 inColor;
layout(location = 3) in uint inElementId;

layout(location = 0) out vec2 fragTexCoord;
layout(location = 1) out vec4 fragColor;
layout(location = 2) flat out uint elementId;

void main() {
    // For now, we'll just use basic orthographic projection
    // This will be replaced with proper projection matrix later
    vec2 clipSpace = inPosition * 2.0 - 1.0;
    gl_Position = vec4(clipSpace, 0.0, 1.0);
    
    fragTexCoord = inTexCoord;
    fragColor = inColor;
    elementId = inElementId;
}