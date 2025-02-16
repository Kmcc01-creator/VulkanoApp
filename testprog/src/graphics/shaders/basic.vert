#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec4 color;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 v_uv;
layout(location = 2) out vec4 v_color;

layout(set = 0, binding = 0) uniform MaterialProperties {
    vec4 albedo;
    float metallic;
    float roughness;
    float ambient_occlusion;
} material;

layout(push_constant) uniform PushConstants {
    mat4 model;
    mat4 view;
    mat4 projection;
} pc;

void main() {
    gl_Position = pc.projection * pc.view * pc.model * vec4(position, 1.0);
    v_normal = mat3(pc.model) * normal;
    v_uv = uv;
    v_color = color * material.albedo;
}