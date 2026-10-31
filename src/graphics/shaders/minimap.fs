#version 330
in vec2 fragTexCoord;
in vec4 fragColor;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform vec2 circleCenter;
uniform float circleRadius;
uniform float screenHeight;
out vec4 finalColor;
void main() {
    vec2 screen = vec2(gl_FragCoord.x, screenHeight-gl_FragCoord.y);
    if (distance(screen,circleCenter)>circleRadius) discard;
    finalColor = texture(texture0,fragTexCoord)*fragColor*colDiffuse;
}
