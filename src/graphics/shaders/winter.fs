#version 330
in vec3 worldPosition;
in vec3 normal;
in vec2 uv;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform vec3 cameraPosition;
out vec4 finalColor;
void main() {
    vec4 base = texture(texture0, uv) * colDiffuse;
    float sunlight = max(dot(normalize(normal), normalize(vec3(-0.3, 0.8, 0.45))), 0.0);
    vec3 linearColor = base.rgb * (0.65 + 0.35 * sunlight);
    // glTF factors are linear. Match a normal display instead of drawing them
    // directly as sRGB bytes (raylib's default shader has no gamma conversion).
    vec3 color = pow(max(linearColor, vec3(0.0)), vec3(1.0 / 2.2));
    float fog = smoothstep(350.0, 1300.0, distance(worldPosition, cameraPosition));
    finalColor = vec4(mix(color, vec3(0.72, 0.83, 0.93), fog), base.a);
}
