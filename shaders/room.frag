#version 330 core

#define MAX_LIGHTS 4

struct PointLight {
    vec3  position;
    vec3  color;
    float constant;
    float linear;
    float quadratic;
};

in vec3 v_frag_pos;
in vec3 v_normal;
in vec2 v_tex_coord;

out vec4 frag_color;

uniform sampler2D u_texture;
uniform bool      u_use_texture;
uniform vec3      u_base_color;

uniform PointLight u_lights[MAX_LIGHTS];
uniform int        u_num_lights;
uniform vec3       u_view_pos;

// ------- Phong helpers -------
vec3 calc_point_light(PointLight light, vec3 normal, vec3 frag_pos, vec3 view_dir) {
    vec3  light_dir   = normalize(light.position - frag_pos);
    float diff        = max(dot(normal, light_dir), 0.0);
    vec3  reflect_dir = reflect(-light_dir, normal);
    float spec        = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    float dist        = length(light.position - frag_pos);
    float attenuation = 1.0 / (light.constant + light.linear * dist + light.quadratic * dist * dist);

    vec3 ambient  = 0.08 * light.color;
    vec3 diffuse  = diff * light.color;
    vec3 specular = 0.3 * spec * light.color;

    return (ambient + diffuse + specular) * attenuation;
}

void main() {
    vec3 base   = u_use_texture ? texture(u_texture, v_tex_coord).rgb : u_base_color;
    vec3 normal = normalize(v_normal);
    vec3 view_dir = normalize(u_view_pos - v_frag_pos);

    vec3 result = vec3(0.0);
    for (int i = 0; i < u_num_lights && i < MAX_LIGHTS; i++) {
        result += calc_point_light(u_lights[i], normal, v_frag_pos, view_dir);
    }

    frag_color = vec4(base * result, 1.0);
}
