#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNormal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;
uniform vec3 aLightPos;

uniform vec3 objectColor;
uniform vec3 lightColor;

out vec3 Color;

void main()
{
    gl_Position = projection * view * model * vec4(aPos, 1.0);

    vec3 pos = vec3(view * model * vec4(aPos, 1.0));
    vec3 normal = mat3(transpose(inverse(view * model))) * aNormal;
    vec3 lightPos = vec3(view * vec4(aLightPos, 1.0));

    float ambientStrength = 0.3;
    float specularStrength = 0.7;

    vec3 ambient = ambientStrength * lightColor;

    vec3 norm = normalize(normal);
    vec3 lightDir = normalize(lightPos - pos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;

    vec3 viewDir = normalize(-pos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 128);
    vec3 specular = specularStrength * spec * lightColor;

    Color = (ambient + diffuse + specular) * objectColor;
}
