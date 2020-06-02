#version 330 core
in vec3 color;
in vec2 textureCoord;

out vec4 FragColor;

uniform sampler2D textureSampler;

void main()
{
    FragColor = texture(textureSampler, textureCoord) * vec4(color, 1.0);
}
