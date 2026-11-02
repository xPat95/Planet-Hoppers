#version 330
in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform vec3 viewForward;
uniform vec3 viewRight;
uniform vec3 viewUp;
uniform vec2 projectionScale;
out vec4 finalColor;
void main() {
    vec2 screen=fragTexCoord*2.0-1.0;
    screen.y=-screen.y;
    vec3 ray=normalize(viewForward+viewRight*screen.x*projectionScale.x*projectionScale.y
        +viewUp*screen.y*projectionScale.y);
    // Original panorama faces Blender +Y, with +Z up; game coordinates are Y-up.
    vec3 b=vec3(ray.x,-ray.z,ray.y);
    vec2 uv=vec2(fract(0.5+atan(b.x,b.y)/6.28318530718),
        0.5-asin(clamp(b.z,-1.0,1.0))/3.14159265359);
    finalColor=vec4(texture(texture0,uv).rgb,1.0);
}
