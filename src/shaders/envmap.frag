#version 330 core

in vec3 Position;
in vec3 Normal;

uniform samplerCube skybox;
uniform vec3 viewPos;

out vec4 FragColor;

void main() {
    float ratio = 1.0 / 1.52;
    vec3 I = normalize(Position - viewPos);
    vec3 R = refract(I, normalize(Normal), ratio);
    FragColor = vec4(texture(skybox, R).rgb, 1.0);
}
