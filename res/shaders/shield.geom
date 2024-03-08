#version 450 core

layout (std140, binding = 0) uniform Common {
    mat4 uScreen;
};

layout (points) in;
layout (triangle_strip, max_vertices = 4) out;

in vec4 vshieldCol[1];
in float vradius[1];

out vec2 uv;
out vec4 shieldCol;

void make_vertex(float x, float y) {
    vec2 corner = vec2(x, y);
    vec4 pos = gl_in[0].gl_Position + vec4(corner, 0.0, 0.0) * (0.5 + vradius[0]);
    gl_Position = uScreen * pos;
    uv = corner;
    shieldCol = vshieldCol[0];
    EmitVertex();
}

void main() {
    // top left
    make_vertex(-1.0, -1.0);
    // top right
    make_vertex(1.0, -1.0);
    // bottom left
    make_vertex(-1.0, 1.0);
    // bottom right
    make_vertex(1.0, 1.0);

    EndPrimitive();
}
