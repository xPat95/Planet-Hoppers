#version 330

uniform vec3 viewForward;
uniform vec3 viewRight;
uniform vec3 viewUp;
uniform vec2 projectionScale;
uniform vec2 resolution;
out vec4 finalColor;

void main() {
    vec2 screen = gl_FragCoord.xy / resolution * 2.0 - 1.0;
    vec3 ray = normalize(viewForward
        + viewRight * screen.x * projectionScale.x * projectionScale.y
        + viewUp * screen.y * projectionScale.y);

    // Smooth functions of direction wrap naturally across the entire sky.
    // Broad interleaved bands replace the old circular gradient patches.
    float blueAxis = dot(ray, normalize(vec3(0.72, 0.24, 0.65)));
    float purpleAxis = dot(ray, normalize(vec3(-0.46, 0.81, 0.36)));
    float blueWave = blueAxis * 3.4 + 0.45 * sin(ray.y * 7.0 + ray.z * 3.0);
    float purpleWave = purpleAxis * 3.8 + 0.38 * sin(ray.x * 6.0 - ray.z * 4.0);
    float blue = exp(-blueWave * blueWave);
    float purple = exp(-purpleWave * purpleWave);
    float wisp = 0.5 + 0.5 * sin(ray.x * 13.0 + ray.y * 8.0 + sin(ray.z * 9.0));
    vec3 color = vec3(0.010, 0.027, 0.090);
    color += vec3(0.006, 0.043, 0.100) * blue * (0.72 + 0.28 * wisp);
    color += vec3(0.031, 0.012, 0.061) * purple * (0.78 + 0.22 * wisp);
    finalColor = vec4(color, 1.0);
}
