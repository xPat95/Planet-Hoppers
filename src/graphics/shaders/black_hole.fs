#version 330
in vec2 fragTexCoord;
in vec4 fragColor;
out vec4 finalColor;
uniform vec2 center;
uniform float radius;
uniform float time;
uniform float strength;
void main() {
    vec2 p = (gl_FragCoord.xy - center) / max(radius, 1.0);
    float r = length(p);
    float disk = length(vec2(p.x, p.y * 3.4));
    float a = atan(p.y * 3.4, p.x);
    float bands = 0.65 + 0.35 * sin(disk * 36.0 - time * 7.0 + a * 4.0);
    float ring = exp(-abs(disk - 1.55) * 2.8) * smoothstep(0.7, 1.05, disk);
    float photon = exp(-abs(r - 0.72) * 36.0);
    float lens = exp(-abs(r - 1.1) * 9.0) * 0.25;
    float intensity = (ring * bands + photon + lens) * strength;
    vec3 hot = mix(vec3(1.0, 0.18, 0.025), vec3(1.0, 0.88, 0.5), min(1.0, intensity));
    float horizon = (1.0 - smoothstep(0.63, 0.67, r)) * strength;
    finalColor = vec4(mix(hot, vec3(0.0), horizon), max(horizon, min(1.0, intensity)));
}
