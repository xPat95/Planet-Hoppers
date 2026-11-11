#version 330
// Bloom, paso 1: se queda solo con lo que brilla (el sol, la lava, las ventanas, los motores), con un umbral suave.
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform float threshold;
out vec4 finalColor;

void main() {
    vec4 s = texture(texture0, fragTexCoord);
    vec3 c = s.rgb;
    float l = max(c.r, max(c.g, c.b));
    float k = smoothstep(threshold, threshold + 0.22, l);
    // Lo marcado con alfa 254/255 (el planeta de Smiling Friends, salvo Satanás) no derrama luz
    if (abs(s.a * 255.0 - 254.0) < 0.5) k = 0.0;
    finalColor = vec4(c * k, 1.0);
}
