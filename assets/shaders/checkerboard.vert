#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) out vec3 texCoord;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};


void main() {
    vec4 worldPosition = Model * vec4(Vertex_Position, 1.0);
    gl_Position = ViewProj * worldPosition;
    texCoord = worldPosition.xyz;
}
