#version 330 core

struct Material {
    sampler2D texture_diffuse[1];
};

uniform Material material;

in GS_OUT {
    vec2 TexCoord;
} fs_in;

out vec4 FragColor;

void main() {
    vec3 color = vec3(texture(material.texture_diffuse[0], fs_in.TexCoord));
    FragColor = vec4(color, 1.0);
}
