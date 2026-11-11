#version 330
// Bloom, paso 3: la escena con el brillo sumado. Se suma sobre todo donde la escena es oscura (alrededor de lo que
// brilla) y apenas sobre lo que ya es claro, así el sol y la nieve no se lavan en blanco y conservan su detalle.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform sampler2D bloom;
uniform float intensity;
out vec4 finalColor;

void main() {
    vec3 c = texture(texture0, fragTexCoord).rgb;
    vec3 b = texture(bloom, fragTexCoord).rgb * intensity;
    float bright = max(c.r, max(c.g, c.b));
    finalColor = vec4(c + b * (1.0 - 0.9 * bright), 1.0);
}
