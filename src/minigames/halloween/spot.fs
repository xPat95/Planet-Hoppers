#version 330
// Minijuego de Halloween: luz de película de terror, en claroscuro. La única luz cálida es el farol de la bruja, en
// alto: muy tenue, sin relleno (lo que no lo mira queda negro),
//  parpadea (`lampPower`) y proyecta sombras de verdad (`shadowMap`, dibujado desde el farol). De atrás de las
// criaturas llega una luz fría y fuerte que solo les marca el borde de la silueta. Todo se hunde en una niebla oscura con la
// distancia; lo que brilla solo (los ojos) atraviesa más la niebla.
in vec3 worldPosition;
in vec3 normal;
in vec2 uv;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform vec3 lightPosition;   // El farol
uniform vec3 cameraPosition;
uniform vec3 emissive;
uniform float ambient;        // Un resto de luz, apenas (0 = negro total)
uniform float outline;
uniform float transportGlow;  // Sin uso en esta escena (el cargador compartido lo pide)
uniform float roughness;
uniform float lightRange;     // Distancia a la que la luz cae a la mitad
uniform float goo;            // 0 a 1: una criatura maldecida se vuelve baba negra y mojada
uniform float hidden;         // 1 = todavía en lo oscuro, invisible del todo (ni los ojos); 0 = ya la agarró la luz
uniform float lampPower;      // El parpadeo del farol (1 = normal, casi 0 = se apaga)
uniform vec3 lampTint;        // Color del farol (cero: el cálido de siempre; en el final, luces de disco)
uniform float fogDensity;
uniform float shadowOn;       // 0 mientras se dibuja el mapa de sombras
uniform sampler2D shadowMap;
uniform mat4 lightVP;
out vec4 finalColor;

const vec3 FOG = vec3(0.0);      // Lo lejos se hunde en negro

float shadowAt(float ndl) {
    vec4 lp = lightVP * vec4(worldPosition, 1.0);
    if (shadowOn < 0.5 || lp.w <= 0.0) return 1.0;
    vec3 c = lp.xyz / lp.w * 0.5 + 0.5;
    if (c.x <= 0.0 || c.x >= 1.0 || c.y <= 0.0 || c.y >= 1.0 || c.z >= 1.0) return 1.0;
    float bias = 0.0012 + 0.004 * (1.0 - ndl);
    vec2 texel = 1.5 / vec2(textureSize(shadowMap, 0));
    float lit = 0.0;
    for (int x = -1; x <= 1; x++) {
        for (int y = -1; y <= 1; y++) {
            lit += (c.z - bias > texture(shadowMap, c.xy + vec2(x, y) * texel).r) ? 0.0 : 1.0;
        }
    }
    return lit / 9.0;
}

void main() {
    vec4 tex = texture(texture0, uv);
    vec4 base = vec4(pow(tex.rgb, vec3(2.2)), tex.a) * colDiffuse;
    base.rgb = mix(base.rgb, vec3(0.006, 0.004, 0.005), goo);
    vec3 n = normalize(normal);
    if (!gl_FrontFacing) n = -n;
    vec3 toLight = lightPosition - worldPosition;
    float dist = length(toLight);
    vec3 l = toLight / max(dist, 1e-4);
    vec3 v = normalize(cameraPosition - worldPosition);
    float fall = 1.0 / (1.0 + pow(dist / lightRange, 2.0));
    vec3 warm = (dot(lampTint, lampTint) > 0.0 ? lampTint : vec3(1.0, 0.58, 0.28)) * 0.11 * lampPower;
    float ndl = max(dot(n, l), 0.0);
    float shade = shadowAt(ndl);
    float gloss = mix(pow(1.0 - clamp(roughness, 0.0, 1.0), 3.0), 0.95, goo);
    float spec = pow(max(dot(n, normalize(l + v)), 0.0), 40.0 + 200.0 * gloss) * (0.12 + 0.6 * gloss) * step(0.0, dot(n, l));
    // El farol, con sombra y sin relleno: solo lo que lo mira de frente se ilumina (claroscuro); la baba negra brilla
    // mojada
    vec3 lit = base.rgb * (ambient + 1.2 * pow(ndl, 1.6) * shade * fall * warm)
             + warm * spec * fall * shade * (0.6 + 3.0 * goo);
    // Luz fría y fuerte de atrás: una línea nítida en el borde de la silueta, del color de la luz (también sobre lo
    // negro, como el demonio), y no en lo que mira para arriba, como el piso
    vec3 rimDir = normalize(vec3(0.85, 0.35, -0.5));   // De atrás y de un costado
    // Corte duro: solo la línea del borde, nada de brillo hacia el frente de la criatura
    float edge = smoothstep(0.48, 0.66, 1.0 - max(dot(n, v), 0.0)) * max(dot(n, rimDir), 0.0) * smoothstep(0.9, 0.55, n.y);
    lit += vec3(0.28, 0.45, 1.0) * edge * 6.0 * (0.6 + 0.4 * (base.r + base.g + base.b));
    // Niebla: lo lejos se hunde en la oscuridad; los ojos la atraviesan más
    float k = exp(-length(cameraPosition - worldPosition) * fogDensity);
    vec3 color = (mix(FOG, lit, k) + emissive * mix(1.0, k, 0.45)) * (1.0 - hidden);
    finalColor = vec4(pow(max(color, vec3(0.0)), vec3(1.0 / 2.2)), base.a);
    finalColor.rgb *= 1.0 + transportGlow;
    if (outline > 0.5) finalColor = vec4(0.0, 0.0, 0.0, 1.0);
}
