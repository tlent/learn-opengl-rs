#version 330 core
layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

in VS_OUT {
    vec2 TexCoord;
} gs_in[];

out GS_OUT {
    vec2 TexCoord;
} gs_out;

uniform float time;

vec3 getNormal();
vec4 explode(vec4 position, vec3 normal);

void main() {
    vec3 normal = getNormal();
    for (int i = 0; i < 3; i++) {
        gs_out.TexCoord = gs_in[i].TexCoord;
        gl_Position = explode(gl_in[i].gl_Position, normal);
        EmitVertex();
    }
    EndPrimitive();
}

vec3 getNormal() {
    vec3 a = vec3(gl_in[0].gl_Position) - vec3(gl_in[1].gl_Position);
    vec3 b = vec3(gl_in[2].gl_Position) - vec3(gl_in[1].gl_Position);
    return normalize(cross(a, b));
}

vec4 explode(vec4 position, vec3 normal) {
    float magnitude = 2.0;
    vec3 d = normal * ((sin(time) + 1.0) / 2.0) * magnitude;
    return position + vec4(d, 0.0);
}
