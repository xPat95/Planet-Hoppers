#version 330
in vec3 worldPosition;
in vec3 normal;
in vec2 uv;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform vec3 cameraPosition;
uniform float alphaCutoff;
uniform int textureIsSrgb;
out vec4 finalColor;
vec3 toLinear(vec3 c) {
    return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(vec3(0.04045), c));
}
vec3 toSrgb(vec3 c) {
    return mix(c * 12.92, 1.055 * pow(c, vec3(1.0 / 2.4)) - 0.055, step(vec3(0.0031308), c));
}
void main() {
    vec4 texel = texture(texture0, uv);
    if (alphaCutoff >= 0.0 && texel.a * colDiffuse.a < alphaCutoff) discard;
    vec3 albedo = (textureIsSrgb == 1 ? toLinear(texel.rgb) : texel.rgb) * colDiffuse.rgb;
    vec3 n = normalize(normal);
    if (!gl_FrontFacing) n = -n;
    float sunlight = max(dot(n, normalize(vec3(-0.4, 0.85, 0.3))), 0.0);
    vec3 color = toSrgb(max(albedo * (0.64 + 0.36 * sunlight), vec3(0.0)));
    float fog = smoothstep(95.0, 180.0, distance(worldPosition, cameraPosition));
    finalColor = vec4(mix(color, vec3(0.62, 0.79, 0.94), fog), 1.0);
}
