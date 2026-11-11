#version 330
// La transición de círculo (iris): la capa de encima se ve solo afuera de un círculo que crece, con el borde apenas
// suave; adentro queda lo de abajo.
in vec2 fragTexCoord;
in vec4 fragColor;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform vec2 center;
uniform float radius;
out vec4 finalColor;

void main() {
    vec4 c = texture(texture0, fragTexCoord) * colDiffuse * fragColor;
    float keep = smoothstep(radius, radius + 3.0, distance(gl_FragCoord.xy, center));
    finalColor = vec4(c.rgb, c.a * keep);
}
