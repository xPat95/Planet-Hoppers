#version 330
// Fotosfera emisiva con granulación y flujo de plasma. Independiente de la luz externa.
in vec3 worldPosition;
in vec3 normal;
in vec2 uv;
uniform vec3 cameraPosition;
uniform vec3 sunCenter;
uniform float time;
// Emisividad, escala del detalle, velocidad de actividad y contraste (centralizados en SunShader).
uniform vec4 surfaceSettings;
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
    float t = time * surfaceSettings.z;
    vec3 p = n * surfaceSettings.y;
    float flow = noise(p * 0.35 + vec3(t, -t * 0.6, t * 0.3));
    p += (flow - 0.5) * 2.5 + vec3(t * 0.4, -t, t * 0.6);
    float g = noise(p) * 0.55 + noise(p * 2.3 + 4.0) * 0.30 + noise(p * 5.1 - t) * 0.15;
    // Al ocupar pocos píxeles, el detalle se integra en vez de producir centelleo.
    float resolved = 1.0 - smoothstep(0.05, 0.45, length(fwidth(p)));
    g = mix(0.55, g, resolved);
    float heat = smoothstep(0.22, 0.78, g);
    vec3 hot = vec3(1.0, 0.98, 0.86);
    vec3 warm = vec3(1.0, 0.48, 0.10);
    // Borde cálido todavía luminoso; no hay sombreado difuso de una esfera mate.
    float limb = mix(0.72, 1.0, smoothstep(0.0, 0.55, mu));
    vec3 plasma = mix(warm, hot, heat);
    float activity = 1.0 + (g - 0.5) * surfaceSettings.w;
    vec3 emission = plasma * surfaceSettings.x * limb * activity;
    // Compresión local de la emisión y salida sRGB: conserva detalle sin quemar a blanco todo el disco.
    vec3 mapped = vec3(1.0) - exp(-emission);
    finalColor = vec4(pow(mapped, vec3(1.0 / 2.2)), 1.0);
}
