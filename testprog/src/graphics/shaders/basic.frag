#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_uv;
layout(location = 2) in vec4 v_color;

layout(set = 0, binding = 0) uniform MaterialProperties {
    vec4 albedo;
    float metallic;
    float roughness;
    float ambient_occlusion;
} material;

layout(location = 0) out vec4 f_color;

void main() {
    vec3 normal = normalize(v_normal);
    vec3 light_dir = normalize(vec3(1.0, 1.0, -1.0));
    
    float diffuse = max(dot(normal, light_dir), 0.0);
    float ambient = 0.1 * material.ambient_occlusion;
    
    vec3 base_color = v_color.rgb * material.albedo.rgb;
    vec3 final_color = base_color * (diffuse + ambient);
    
    // Simple metallic-roughness approximation
    float fresnel = 0.04 + (1.0 - 0.04) * (1.0 - max(dot(normal, light_dir), 0.0));
    vec3 metal_color = mix(vec3(fresnel), base_color, material.metallic);
    
    // Roughness affects specular highlight
    float roughness_factor = 1.0 - material.roughness;
    vec3 reflect_dir = reflect(-light_dir, normal);
    float spec = pow(max(dot(reflect_dir, normalize(vec3(0.0, 0.0, -1.0))), 0.0), 16.0 * roughness_factor);
    
    final_color = mix(final_color, metal_color + spec * vec3(1.0), material.metallic);
    
    f_color = vec4(final_color, v_color.a * material.albedo.a);
}