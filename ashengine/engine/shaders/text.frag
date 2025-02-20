#version 450

layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) in vec4 fragColor;
layout(location = 2) flat in uint elementId;

layout(location = 0) out vec4 outColor;

layout(binding = 1) uniform sampler2D fontTexture;

// SDF parameters
const float smoothing = 0.125;
const float thickness = 0.5;

void main() {
    // Sample the signed distance field
    float distance = texture(fontTexture, fragTexCoord).r;
    
    // Calculate alpha based on the distance field
    float alpha = smoothstep(thickness - smoothing, thickness + smoothing, distance);
    
    // Apply color with calculated alpha
    outColor = vec4(fragColor.rgb, fragColor.a * alpha);
}