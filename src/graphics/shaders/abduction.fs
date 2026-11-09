#version 330
in vec2 uv;
in vec3 normal;
in vec4 color;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
out vec4 finalColor;
void main() {
    vec4 base = texture(texture0,uv) * colDiffuse * color;
    float key = max(dot(normalize(normal),normalize(vec3(-0.4,0.8,1.0))),0.0);
    float rim = pow(1.0-abs(normalize(normal).z),2.0);
    finalColor = vec4(base.rgb*(0.72+0.28*key)+vec3(0.10,0.20,0.25)*rim,base.a);
}
