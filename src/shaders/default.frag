#version 330 core

struct Material {
    sampler2D diffuse;
    sampler2D specular;
    float shininess;
};

struct DirectionalLight {
    vec3 direction;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct PointLight {
    vec3 position;

    float constant;
    float linear;
    float quadratic;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

struct SpotLight {
    vec3 position;
    vec3 direction;

    float constant;
    float linear;
    float quadratic;

    float innerCutoff;
    float outerCutoff;

    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

uniform vec3 viewPos;
uniform Material material;

uniform DirectionalLight directionalLight;
#define POINT_LIGHT_COUNT 4
uniform PointLight pointLights[POINT_LIGHT_COUNT];
uniform SpotLight spotLight;

in vec3 Normal;
in vec3 FragPos;
in vec2 TexCoord;

out vec4 FragColor;

vec3 calculateDirectionalLighting(DirectionalLight l, vec3 normal, vec3 viewDir);
vec3 calculatePointLighting(PointLight light, vec3 normal, vec3 viewDir, vec3 fragPos);
vec3 calculateSpotLighting(SpotLight light, vec3 normal, vec3 viewDir, vec3 fragPos);

void main() {
    vec3 normal = normalize(Normal);
    vec3 viewDir = normalize(viewPos - FragPos);

    vec3 color = calculateDirectionalLighting(directionalLight, normal, viewDir);
    for (int i = 0; i < POINT_LIGHT_COUNT; i++) {
        color += calculatePointLighting(pointLights[i], normal, viewDir, FragPos);
    }
    color += calculateSpotLighting(spotLight, normal, viewDir, FragPos);

    FragColor = vec4(color, 1.0);
}

vec3 calculateDirectionalLighting(DirectionalLight light, vec3 normal, vec3 viewDir) {
    vec3 lightDir = normalize(-light.direction);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    // combine results
    vec3 ambient  = light.ambient  * vec3(texture(material.diffuse, TexCoord));
    vec3 diffuse  = light.diffuse  * diff * vec3(texture(material.diffuse, TexCoord));
    vec3 specular = light.specular * spec * vec3(texture(material.specular, TexCoord));
    return (ambient + diffuse + specular);
}

vec3 calculatePointLighting(PointLight light, vec3 normal, vec3 viewDir, vec3 fragPos) {
    vec3 lightDir = normalize(light.position - fragPos);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    // combine results
    vec3 ambient = light.ambient * vec3(texture(material.diffuse, TexCoord));
    vec3 diffuse = light.diffuse * diff * vec3(texture(material.diffuse, TexCoord));
    vec3 specular = light.specular * spec * vec3(texture(material.specular, TexCoord));
    // attenuation
    float distance    = length(light.position - fragPos);
    float attenuation = 1.0 / (
        light.quadratic * distance * distance
        + light.linear * distance
        + light.constant
    );
    return attenuation * (ambient + diffuse + specular);
}

vec3 calculateSpotLighting(SpotLight light, vec3 normal, vec3 viewDir, vec3 fragPos) {
    vec3 lightDir = normalize(light.position - fragPos);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
    // combine results
    vec3 ambient = light.ambient * vec3(texture(material.diffuse, TexCoord));
    vec3 diffuse = light.diffuse * diff * vec3(texture(material.diffuse, TexCoord));
    vec3 specular = light.specular * spec * vec3(texture(material.specular, TexCoord));
    // attenuation
    float distance  = length(light.position - fragPos);
    float attenuation = 1.0 / (
        light.quadratic * distance * distance
        + light.linear * distance
        + light.constant
    );
    // intensity
    float theta = dot(lightDir, normalize(-light.direction));
    float epsilon = light.innerCutoff - light.outerCutoff;
    float intensity = clamp((theta - light.outerCutoff) / epsilon, 0.0, 1.0);
    return attenuation * intensity * (ambient + diffuse + specular);
}
