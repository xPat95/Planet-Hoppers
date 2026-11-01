#version 330
// Superficie del Sol: granos que hierven despacio y el borde más anaranjado y oscuro que el centro.
in vec3 worldPosition;
in vec3 normal;
in vec2 uv;
uniform vec3 cameraPosition;
uniform vec3 sunCenter;
uniform float time;
out vec4 finalColor;

float hash(vec3 p) {
    p = fract(p * 0.3183099 + 0.1);
    p *= 17.0;
    return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
}

float noise(vec3 x) {
    vec3 i = floor(x);
    vec3 f = fract(x);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(mix(hash(i), hash(i + vec3(1, 0, 0)), f.x), mix(hash(i + vec3(0, 1, 0)), hash(i + vec3(1, 1, 0)), f.x), f.y),
               mix(mix(hash(i + vec3(0, 0, 1)), hash(i + vec3(1, 0, 1)), f.x), mix(hash(i + vec3(0, 1, 1)), hash(i + vec3(1, 1, 1)), f.x), f.y), f.z);
}

void main() {
    vec3 n = normalize(worldPosition - sunCenter);
    vec3 v = normalize(cameraPosition - worldPosition);
    float mu = max(dot(n, v), 0.0);
    vec3 p = n * 6.0 + vec3(0.0, time * 0.05, time * 0.03);
    float g = noise(p) * 0.6 + noise(p * 2.3 + 4.0) * 0.3 + noise(p * 5.1 - time * 0.1) * 0.1;
    vec3 hot = vec3(1.0, 0.97, 0.84);
    vec3 warm = vec3(1.0, 0.72, 0.25);
    vec3 edge = vec3(0.95, 0.38, 0.08);
    vec3 c = mix(edge, mix(warm, hot, smoothstep(0.35, 1.0, mu)), smoothstep(0.0, 0.45, mu));
    c *= 0.82 + 0.3 * g;
    finalColor = vec4(c, 1.0);
}
