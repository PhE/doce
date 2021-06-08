#version 450

layout(location = 0) out vec4 o_Target;
layout(location = 1) in vec3 texCoord;

layout(set = 2, binding = 0) uniform CheckerboardMaterial_first_color {
    vec4 first_color;
};

layout(set = 3, binding = 0) uniform CheckerboardMaterial_second_color {
    vec4 second_color;
};


void main() {
    bool isEven = mod(round(texCoord.x) + round(texCoord.y) + round(texCoord.z), 2.0) == 0.0;
    o_Target = isEven ? first_color : second_color;
}
