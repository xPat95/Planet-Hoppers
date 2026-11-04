#version 330
// Planetas y naves: la única luz es el Sol, en el centro del sistema.
in vec3 worldPosition;
in vec3 normal;
in vec2 uv;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform vec3 lightPosition;
uniform vec3 cameraPosition;
uniform vec3 emissive;
uniform float ambient;
uniform float outline;   // 1 = contorno de caricatura: negro liso; 2 = personaje plano (sin luz)
uniform float transportGlow;
uniform float transportDissolve;
uniform float roughness; // Del material (glTF): 0 = brilla como vidrio mojado, 1 = mate
uniform float studio;    // 1 = hangar: luces de estudio (clave cálida, contorno frío y una de abajo); 0 = espacio
out vec4 finalColor;
void main() {
    // Las imágenes vienen en sRGB y los factores de glTF son lineales: todo se pasa a lineal antes de iluminar
    vec4 tex = texture(texture0, uv);
    vec4 base = vec4(pow(tex.rgb, vec3(2.2)), tex.a) * colDiffuse;
    // Algunas mallas tienen normales invertidas: se usa la cara que mira a la cámara
    vec3 n = normalize(normal);
    if (!gl_FrontFacing) n = -n;
    vec3 l = normalize(lightPosition - worldPosition);
    vec3 v = normalize(cameraPosition - worldPosition);
    float ndl = dot(n, l);
    // Terminador suave entre el día y la noche
    float day = clamp((ndl + 0.12) / 1.12, 0.0, 1.0);
    // Brillo según el material: lo pulido (estrellas infladas, ojos, sangre) tiene un reflejo chico y fuerte;
    // lo mate, uno ancho y suave
    float gloss = pow(1.0 - clamp(roughness, 0.0, 1.0), 3.0);
    float shininess = 40.0 + 200.0 * gloss;
    float specStrength = 0.12 + 0.6 * gloss;
    float spec = pow(max(dot(n, normalize(l + v)), 0.0), shininess) * specStrength * step(0.0, ndl);
    float rim = pow(1.0 - max(dot(n, v), 0.0), 3.0);
    vec3 sun = vec3(1.0, 0.94, 0.82);
    vec3 night = vec3(0.45, 0.52, 0.85) * ambient;
    vec3 lit = base.rgb * (night + sun * day) + sun * spec + emissive
             + rim * day * 0.3 * sun * base.rgb      // borde iluminado del lado del Sol
             + rim * 0.05 * vec3(0.35, 0.45, 1.0);   // contorno azulado tenue del lado oscuro
    if (studio > 0.5) {
        // Luces de estudio del hangar, fijas en el mundo (la nave gira debajo): como en las capturas de Blender
        vec3 keyL = normalize(vec3(0.1, 0.8, 0.9));      // Clave: arriba, del lado de la cámara, cálida
        vec3 rimL = normalize(vec3(-0.6, 0.45, -0.9));   // Contorno: de atrás, fría
        vec3 underL = normalize(vec3(0.3, -1.0, 0.4));   // De abajo: naranja, suave
        float keyD = max(dot(n, keyL), 0.0);
        float keySpec = pow(max(dot(n, normalize(keyL + v)), 0.0), shininess) * specStrength * step(0.0, dot(n, keyL));
        float rimD = max(dot(n, rimL), 0.0) * pow(1.0 - max(dot(n, v), 0.0), 1.5);
        lit += base.rgb * keyD * vec3(1.0, 0.88, 0.78) * 0.85
             + keySpec * vec3(1.0, 0.95, 0.9) * 1.4
             + rimD * vec3(0.45, 0.55, 1.0) * 0.9
             + base.rgb * max(dot(n, underL), 0.0) * vec3(1.0, 0.45, 0.2) * 0.3;
    }
    // Los colores de glTF son lineales: se pasan a la pantalla (raylib no corrige la gamma)
    finalColor = vec4(pow(max(lit, vec3(0.0)), vec3(1.0 / 2.2)), base.a);
    if (outline > 1.5) {
        // Personaje de caricatura: los colores planos de su dibujo, sin luz
        finalColor = vec4(tex.rgb, 1.0);
    } else if (outline > 0.5) {
        finalColor = vec4(0.02, 0.02, 0.03, 1.0);
    }
    vec3 fromCenter = worldPosition - vec3(0.0, 1.3, 0.0);
    float reach = clamp(transportGlow * 0.45 / (1.0 + dot(fromCenter, fromCenter) * 0.38), 0.0, 0.9);
    finalColor.rgb = mix(finalColor.rgb,
        finalColor.rgb * vec3(0.54, 0.82, 1.05) + vec3(0.05, 0.13, 0.24), reach);
    if (transportDissolve > 0.001) {
        float grain = fract(sin(dot(floor(worldPosition * 21.0), vec3(12.9898, 78.233, 37.719))) * 43758.5453);
        if (grain < transportDissolve) discard;
        float edge = 1.0 - smoothstep(0.0, 0.13, grain - transportDissolve);
        finalColor.rgb = mix(finalColor.rgb, vec3(0.40, 0.86, 1.0), edge * 0.75);
    }
}
