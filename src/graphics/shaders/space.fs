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
uniform vec3 planetFill; // Relleno suave de cada planeta; cero para naves y estación
uniform vec3 planetKeyPosition;
uniform vec4 planetKey; // rgb = color, a = fuerza; cero para naves y estación
uniform float planetKeyRange;
uniform float planetWet;
uniform float wetSurface; // 1 = rojo mojado, 2 = sangre/grietas negras
uniform float planetToon; // 1 = bandas de luz y borde de tinta para Smiling Friends
uniform float noBloom;    // 1 = no derrama luz en el bloom (se marca con alfa 254/255, invisible)
out vec4 finalColor;
void main() {
    // Las imágenes vienen en sRGB y los factores de glTF son lineales: todo se pasa a lineal antes de iluminar
    vec4 tex = texture(texture0, uv);
    vec4 base = vec4(pow(tex.rgb, vec3(2.2)), tex.a) * colDiffuse;
    if (planetWet > 0.5 && wetSurface > 0.5 && wetSurface < 1.5) {
        base.rgb *= vec3(1.10, 0.76, 0.76);
    }
    // Algunas mallas tienen normales invertidas: se usa la cara que mira a la cámara
    vec3 n = normalize(normal);
    if (!gl_FrontFacing) n = -n;
    vec3 l = normalize(lightPosition - worldPosition);
    vec3 v = normalize(cameraPosition - worldPosition);
    float ndl = dot(n, l);
    // Terminador suave entre el día y la noche
    float day = smoothstep(-0.08, 0.82, ndl);
    // Brillo según el material: lo pulido (estrellas infladas, ojos, sangre) tiene un reflejo chico y fuerte;
    // lo mate, uno ancho y suave
    float gloss = pow(1.0 - clamp(roughness, 0.0, 1.0), 3.0);
    float shininess = 40.0 + 200.0 * gloss;
    float specStrength = 0.12 + 0.6 * gloss;
    float spec = pow(max(dot(n, normalize(l + v)), 0.0), shininess) * specStrength * step(0.0, ndl);
    float rim = pow(1.0 - max(dot(n, v), 0.0), 3.0);
    vec3 sun = vec3(1.0, 0.94, 0.82);
    float sunDistance = length(lightPosition - worldPosition);
    float sunReach = 1.0 / (1.0 + pow(max(sunDistance - 45.0, 0.0) / 500.0, 1.6));
    float spaceReach = mix(sunReach, 1.0, step(0.5, studio));
    vec3 night = vec3(0.30, 0.36, 0.58) * ambient;
    vec3 lit = base.rgb * (night + planetFill + sun * day * 0.90 * spaceReach) + sun * spec * 0.72 * spaceReach + emissive
             + rim * day * 0.11 * spaceReach * sun * base.rgb
             + rim * (1.0 - day) * 0.025 * vec3(0.38, 0.48, 0.82);
    if (planetToon > 0.5) {
        // Mantener los colores dibujados: dos tonos, sin brillo especular ni degradado de borde.
        float band = 0.72 + 0.28 * smoothstep(0.10, 0.17, ndl);
        float ink = 1.0 - 0.22 * smoothstep(0.78, 0.92, rim);
        lit = base.rgb * band * ink + emissive;
    }
    if (planetKey.a > 0.0) {
        vec3 delta = planetKeyPosition - worldPosition;
        float keyDistance2 = dot(delta, delta);
        vec3 keyDirection = normalize(delta);
        float keyDiffuse = max(dot(n, keyDirection), 0.0);
        float keyFalloff = 1.0 / (1.0 + keyDistance2 / max(planetKeyRange * planetKeyRange, 1.0));
        lit += base.rgb * planetKey.rgb * planetKey.a * keyDiffuse * keyFalloff;
        if (planetWet > 0.5 && wetSurface > 0.5) {
            float facing = max(dot(n, v), 0.0);
            float fresnel = pow(1.0 - facing, 3.0);
            float reflection = max(dot(n, normalize(keyDirection + v)), 0.0);
            float broad = pow(reflection, 45.0);
            float sharp = pow(reflection, 210.0);
            float blackLiquid = step(1.5, wetSurface);
            lit += planetKey.rgb * (broad * mix(0.24, 0.55, blackLiquid)
                + sharp * mix(2.8, 4.2, blackLiquid)) * keyFalloff;
            float sunReflection = pow(max(dot(n, normalize(l + v)), 0.0), 90.0);
            lit += vec3(1.0, 0.87, 0.78) * sunReflection * mix(1.1, 2.2, blackLiquid) * spaceReach;
            // Reflejo largo de una fuente amplia: visible incluso sobre sangre casi negra.
            vec3 stripDirection = normalize(vec3(-0.65, 0.85, 0.25));
            float strip = pow(max(dot(n, normalize(stripDirection + v)), 0.0), 145.0);
            lit += vec3(0.91, 0.94, 1.0) * strip * mix(0.42, 1.45, blackLiquid);
            // Dos formas de luz reflejadas sobre el líquido; sin textura, cubemap ni pasada extra.
            vec3 reflected = reflect(-v, n);
            float boxA = smoothstep(0.94, 0.985, dot(reflected, normalize(vec3(-0.55, 0.72, 0.42))));
            float boxB = smoothstep(0.975, 0.995, dot(reflected, normalize(vec3(0.58, 0.62, -0.52))));
            lit += vec3(0.78, 0.88, 1.0) * (boxA * 0.19 + boxB * 0.13) * mix(1.0, 1.55, blackLiquid);
            lit += vec3(0.25, 0.35, 0.46) * fresnel * mix(0.18, 0.32, blackLiquid);
        }
    }
    if (studio > 0.5) {
        // Luces de estudio del hangar, fijas en el mundo (la nave gira debajo): como en las capturas de Blender
        vec3 keyL = normalize(vec3(0.1, 0.8, 0.9));      // Clave: arriba, del lado de la cámara, cálida
        vec3 rimL = normalize(vec3(-0.6, 0.45, -0.9));   // Contorno: de atrás, fría
        vec3 underL = normalize(vec3(0.3, -1.0, 0.4));   // De abajo: naranja, suave
        float keyD = max(dot(n, keyL), 0.0);
        float keySpec = pow(max(dot(n, normalize(keyL + v)), 0.0), shininess) * specStrength * step(0.0, dot(n, keyL));
        float rimD = max(dot(n, rimL), 0.0) * pow(1.0 - max(dot(n, v), 0.0), 1.5);
        lit += base.rgb * keyD * vec3(0.84, 0.94, 1.0) * 1.05
             + keySpec * vec3(0.83, 0.96, 1.0) * 1.25
             + rimD * vec3(0.63, 0.47, 0.94) * 0.72
             + base.rgb * max(dot(n, underL), 0.0) * vec3(0.43, 0.70, 1.0) * 0.18;
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
    if (noBloom > 0.5) finalColor.a = min(finalColor.a, 254.0 / 255.0);
}
