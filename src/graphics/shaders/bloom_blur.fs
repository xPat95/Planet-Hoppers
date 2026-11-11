#version 330
// Bloom, paso 2: desenfoque gaussiano en una dirección (`dir`, un texel; se usa dos veces, horizontal y vertical).
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec2 dir;
out vec4 finalColor;

void main() {
    const float w[5] = float[](0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);
    vec3 c = texture(texture0, fragTexCoord).rgb * w[0];
    for (int i = 1; i < 5; i++) {
        c += texture(texture0, fragTexCoord + dir * float(i) * 1.5).rgb * w[i];
        c += texture(texture0, fragTexCoord - dir * float(i) * 1.5).rgb * w[i];
    }
    finalColor = vec4(c, 1.0);
}
