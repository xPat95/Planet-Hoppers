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
    float spec = pow(max(dot(n, normalize(l + v)), 0.0), 40.0) * 0.12 * step(0.0, ndl);
    float rim = pow(1.0 - max(dot(n, v), 0.0), 3.0);
    vec3 sun = vec3(1.0, 0.94, 0.82);
    vec3 night = vec3(0.45, 0.52, 0.85) * ambient;
    vec3 lit = base.rgb * (night + sun * day) + sun * spec + emissive
             + rim * day * 0.3 * sun * base.rgb      // borde iluminado del lado del Sol
             + rim * 0.05 * vec3(0.35, 0.45, 1.0);   // contorno azulado tenue del lado oscuro
    // Los colores de glTF son lineales: se pasan a la pantalla (raylib no corrige la gamma)
    finalColor = vec4(pow(max(lit, vec3(0.0)), vec3(1.0 / 2.2)), base.a);
    if (outline > 1.5) {
        // Personaje de caricatura: los colores planos de su dibujo, sin luz
        finalColor = vec4(tex.rgb, 1.0);
    } else if (outline > 0.5) {
        finalColor = vec4(0.02, 0.02, 0.03, 1.0);
    }
}
