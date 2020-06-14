#version 330 core

in vec2 TexCoord;

uniform sampler2D tex;

out vec4 FragColor;

void main() {
    FragColor = vec4(vec3(gl_FragCoord.z), 1.0);
}
