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
    vec3 warped = ray + 0.09 * vec3(
        sin(ray.y * 13.0 + ray.z * 8.0),
        sin(ray.z * 11.0 - ray.x * 9.0),
        sin(ray.x * 12.0 + ray.y * 7.0));
    float cloud = 0.5 + 0.5 * sin(warped.x * 17.0 + warped.z * 11.0)
        * sin(warped.y * 15.0 - warped.z * 9.0);
    float fine = 0.5 + 0.5 * sin(warped.x * 39.0 + warped.y * 29.0 + sin(warped.z * 23.0));
    float density = smoothstep(0.32, 0.82, cloud * 0.72 + fine * 0.28);
    vec3 color = vec3(0.005, 0.013, 0.043);
    color += vec3(0.007, 0.038, 0.087) * blue * (0.18 + 0.82 * density);
    color += vec3(0.028, 0.011, 0.060) * purple * (0.17 + 0.83 * density);
    // Azul mas presente en la parte alta de la pantalla, con una transicion amplia.
    float upperBlue = smoothstep(0.02, 0.95, screen.y);
    color += vec3(0.013, 0.050, 0.125) * upperBlue;
    finalColor = vec4(color, 1.0);
}
