#version 330 core
layout (triangles) in;
layout (line_strip, max_vertices = 6) out;

in VS_OUT {
    vec3 Normal;
} gs_in[];

uniform mat4 projection;

const float MAGNITUDE = 0.4;

void main() {
    for (int i = 0; i < 3; i++) {
        vec4 p = gl_in[i].gl_Position;
        gl_Position = projection * p;
        EmitVertex();
        vec3 offset = gs_in[i].Normal * MAGNITUDE;
        gl_Position = projection * (p + vec4(offset, 0.0));
        EmitVertex();
        EndPrimitive();
    }
}
