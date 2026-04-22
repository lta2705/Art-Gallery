#version 330 core

#define MAX_LIGHTS 4

struct PointLight {
    vec3  position;
    vec3  color;
    float constant;
    float linear;
    float quadratic;
};

struct SpotLight {
    vec3 position;
    vec3 direction;
    vec3 color;
    float cutOff;
    float outerCutOff;
    
    float constant;
    float linear;
    float quadratic;
};

in vec3 v_frag_pos;
in vec3 v_normal;
in vec2 v_tex_coord;
in vec4 v_frag_pos_light_space;

out vec4 frag_color;

uniform sampler2D u_texture;
uniform bool      u_use_texture;
uniform vec3      u_base_color;

uniform PointLight u_lights[MAX_LIGHTS];
uniform int        u_num_lights;

uniform SpotLight  u_spot_light;
uniform bool       u_use_spotlight;

uniform vec3       u_view_pos;

uniform sampler2D u_shadow_map;

// ------- Shadow Calculation -------
float calc_shadow(vec4 frag_pos_light_space, vec3 normal, vec3 light_dir) {
    vec3 proj_coords = frag_pos_light_space.xyz / frag_pos_light_space.w;
    proj_coords = proj_coords * 0.5 + 0.5;
    
    if(proj_coords.z > 1.0)
        return 0.0;
        
    float closest_depth = texture(u_shadow_map, proj_coords.xy).r; 
    float current_depth = proj_coords.z;
    
    // Bias to prevent shadow acne
    float bias = max(0.005 * (1.0 - dot(normal, light_dir)), 0.0005);
    
    // PCF (Percentage-Closer Filtering)
    float shadow = 0.0;
    vec2 texel_size = 1.0 / textureSize(u_shadow_map, 0);
    for(int x = -1; x <= 1; ++x) {
        for(int y = -1; y <= 1; ++y) {
            float pcf_depth = texture(u_shadow_map, proj_coords.xy + vec2(x, y) * texel_size).r; 
            shadow += current_depth - bias > pcf_depth ? 1.0 : 0.0;        
        }    
    }
    shadow /= 9.0;
    
    return shadow;
}

// ------- Blinn-Phong helpers -------
vec3 calc_point_light(PointLight light, vec3 normal, vec3 frag_pos, vec3 view_dir) {
    vec3  light_dir   = normalize(light.position - frag_pos);
    float diff        = max(dot(normal, light_dir), 0.0);
    
    // Blinn-Phong: Use halfway vector
    vec3 halfway_dir  = normalize(light_dir + view_dir);
    float spec        = pow(max(dot(normal, halfway_dir), 0.0), 32.0);
    
    float dist        = length(light.position - frag_pos);
    float attenuation = 1.0 / (light.constant + light.linear * dist + light.quadratic * dist * dist);

    vec3 ambient  = 0.05 * light.color;
    vec3 diffuse  = diff * light.color;
    vec3 specular = 0.3 * spec * light.color;

    return (ambient + diffuse + specular) * attenuation;
}

vec3 calc_spot_light(SpotLight light, vec3 normal, vec3 frag_pos, vec3 view_dir) {
    vec3 light_dir = normalize(light.position - frag_pos);
    float diff = max(dot(normal, light_dir), 0.0);
    
    vec3 halfway_dir = normalize(light_dir + view_dir);
    float spec = pow(max(dot(normal, halfway_dir), 0.0), 64.0);
    
    float dist = length(light.position - frag_pos);
    float attenuation = 1.0 / (light.constant + light.linear * dist + light.quadratic * dist * dist);
    
    float theta = dot(light_dir, normalize(-light.direction));
    float epsilon = light.cutOff - light.outerCutOff;
    float intensity = clamp((theta - light.outerCutOff) / epsilon, 0.0, 1.0);
    
    vec3 ambient = 0.02 * light.color;
    vec3 diffuse = light.color * diff * intensity;
    vec3 specular = light.color * spec * intensity;
    
    float shadow = calc_shadow(v_frag_pos_light_space, normal, light_dir);
    
    return (ambient + (1.0 - shadow) * (diffuse + specular)) * attenuation;
}

void main() {
    vec3 base   = u_use_texture ? texture(u_texture, v_tex_coord).rgb : u_base_color;
    vec3 normal = normalize(v_normal);
    vec3 view_dir = normalize(u_view_pos - v_frag_pos);

    vec3 result = vec3(0.0);
    for (int i = 0; i < u_num_lights && i < MAX_LIGHTS; i++) {
        result += calc_point_light(u_lights[i], normal, v_frag_pos, view_dir);
    }
    
    if (u_use_spotlight) {
        result += calc_spot_light(u_spot_light, normal, v_frag_pos, view_dir);
    } else {
        // Just ambient if no lights
        if (u_num_lights == 0) result += base * 0.1;
    }

    frag_color = vec4(base * result, 1.0);
}
