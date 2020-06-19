#version 330 core
layout (points) in;
layout (triangle_strip, max_vertices = 5) out;

in VS_OUT {
    vec3 color;
} gs_in[];

out vec3 Color;

void main() {
    Color = gs_in[0].color;
    vec4 p = gl_in[0].gl_Position;
    float size = 0.2;
    gl_Position = p + vec4(-size, -size, 0.0, 0.0);
    EmitVertex();
    gl_Position = p + vec4(size, -size, 0.0, 0.0);
    EmitVertex();
    gl_Position = p + vec4(-size, size, 0.0, 0.0);
    EmitVertex();
    gl_Position = p + vec4(size, size, 0.0, 0.0);
    EmitVertex();
    Color = vec3(1.0);
    gl_Position = p + vec4(0.0, size * 2, 0.0, 0.0);
    EmitVertex();
    EndPrimitive();
}
