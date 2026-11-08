#version 330
// Minijuego de Halloween: ojo de pez fuerte, a pantalla completa (como el dibujo de referencia de Cami). La escena se
// dibuja con una cámara muy abierta en una textura cuadrada; acá cada punto de la pantalla se lee según su ángulo con
// una proyección equisólida (r = sen(θ/2)): el centro se agranda mucho y los bordes se aprietan y se curvan, así lo
// que está cerca y adelante se infla hacia la cámara. Las esquinas de la pantalla llegan al ángulo máximo.
// Encima, grano de película y bordes oscuros; con un susto (`hit`), un sacudón, los colores se separan y hay estática.
// `screen_from_scene` en render.rs hace la cuenta inversa para ubicar el HUD sobre lo que se ve.
in vec2 fragTexCoord;
in vec4 fragColor;
uniform sampler2D texture0;
uniform float aspect;    // Ancho / alto de la pantalla
uniform float fovHalf;   // Ángulo en las esquinas de la pantalla (radianes); la textura cubre lo mismo de lado a lado
uniform float time;
uniform float hit;       // 0 a 1: el susto
uniform float seeThrough; // 1: lo que no tiene nada queda transparente (se ve lo que hay atrás en la pantalla)
out vec4 finalColor;

// Ruido sin patrones (el de seno dibuja rayas en diagonal)
float hash(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

vec2 lens(vec2 st, float stretch) {
    vec2 p = (st * 2.0 - 1.0) * vec2(aspect, 1.0) / sqrt(aspect * aspect + 1.0);
    float r = length(p);
    vec2 dir = r > 1e-5 ? p / r : vec2(0.0);
    float theta = 2.0 * asin(clamp(r * sin(fovHalf * 0.5), 0.0, 1.0));
    return 0.5 + 0.5 * dir * (tan(theta) / tan(fovHalf)) * stretch;
}

void main() {
    vec2 st = fragTexCoord + hit * vec2(sin(time * 91.0), cos(time * 77.0)) * 0.012;
    float split = hit * 0.035;
    vec3 col = vec3(texture(texture0, lens(st, 1.0 + split)).r,
                    texture(texture0, lens(st, 1.0)).g,
                    texture(texture0, lens(st, 1.0 - split)).b);
    vec2 p = (fragTexCoord * 2.0 - 1.0) * vec2(aspect, 1.0) / sqrt(aspect * aspect + 1.0);
    float r = length(p);
    float grain = hash(floor(fragTexCoord * vec2(960.0, 540.0)) + floor(time * 24.0) * 7.13);
    // El grano solo sobre lo que tiene luz: el negro queda negro
    col += (grain - 0.5) * 0.045 * smoothstep(0.0, 0.08, max(col.r, max(col.g, col.b)));
    col = mix(col, vec3(grain), hit * 0.4 * step(0.45, hash(vec2(floor(time * 30.0), 3.0))));
    float vignette = smoothstep(1.0, 0.4, r);
    float alpha = mix(1.0, texture(texture0, lens(st, 1.0)).a, seeThrough);
    finalColor = vec4(max(col, vec3(0.0)) * mix(mix(0.15, 1.0, vignette), 1.0, seeThrough), alpha);
}
