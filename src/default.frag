#version 330 core
in vec3 color;
in vec2 textureCoord;

out vec4 FragColor;

uniform sampler2D textureSampler;
uniform sampler2D textureSampler2;

void main()
{
    FragColor = mix(texture(textureSampler, textureCoord), texture(textureSampler2, textureCoord), 0.2);
}
