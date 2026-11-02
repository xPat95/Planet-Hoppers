#version 330

in vec2 fragTexCoord;
in vec4 fragColor;
uniform vec2 resolution;
uniform float time;
out vec4 finalColor;

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 345.45));
    p += dot(p, p + 34.345);
    return fract(p.x * p.y);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), f.x),
               mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), f.x), f.y);
}

float cloud(vec2 p) {
    float n = 0.0;
    n += 0.55 * noise(p);
    n += 0.28 * noise(p * 2.1 + 17.0);
    n += 0.17 * noise(p * 4.3 - 9.0);
    return n;
}

void main() {
    vec2 p = (gl_FragCoord.xy - resolution * 0.5) / resolution.y;
    float r = length(p);
    float angle = atan(p.y, p.x);
    float depth = log(r + 0.055);

    // Constant-speed travel in log radius makes dust and light bands converge on the center.
    float flow = depth * 2.7 + time * 0.38;
    // Circular coordinates meet cleanly at the left edge of atan's range.
    vec2 radial = p / max(r, 0.001);
    float grain = cloud(radial * (2.5 + depth * 0.30) + vec2(flow * 0.33, flow * 0.25));
    float wisps = cloud(radial * (4.7 - depth * 0.25) + vec2(-flow * 0.42, flow * 0.38 + 7.0));
    float arms = pow(0.5 + 0.5 * cos(angle * 4.0 + depth * 7.0 + time * 1.0 + grain * 2.8), 4.0);
    float bands = pow(0.5 + 0.5 * cos(depth * 17.0 + time * 2.4 + wisps * 2.0), 8.0);
    float halo = 1.0 - smoothstep(0.10, 1.25, r);
    float dust = halo * smoothstep(0.28, 0.75, grain) * (0.40 + 0.60 * wisps);
    float filaments = halo * arms * (0.35 + 0.65 * grain);
    float core = exp(-r * r * 22.0);

    // The hue cycles slowly through deep blue, sky blue, and violet.
    float purple = 0.5 + 0.5 * sin(time * 0.26 + depth * 0.35);
    float sky = 0.5 + 0.5 * sin(time * 0.26 + 2.2 + angle * 2.0);
    vec3 nebula = mix(vec3(0.020, 0.19, 0.52), vec3(0.37, 0.075, 0.58), purple);
    nebula = mix(nebula, vec3(0.08, 0.54, 0.82), sky * 0.55);

    vec3 color = vec3(0.003, 0.009, 0.029);
    color += nebula * halo * (0.12 + 0.70 * dust + 0.70 * filaments);
    color += mix(vec3(0.06, 0.32, 0.62), vec3(0.26, 0.17, 0.62), purple)
             * halo * bands * (0.25 + 0.75 * grain) * 0.42;
    color += mix(vec3(0.20, 0.65, 0.95), vec3(0.60, 0.30, 0.90), purple)
             * core * 0.38;
    finalColor = vec4(clamp(color, 0.0, 1.0), 1.0);
}
