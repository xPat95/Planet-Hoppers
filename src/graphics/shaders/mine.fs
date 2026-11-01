#version 330
in vec3 worldPosition;
in vec3 normal;
in vec2 uv;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform vec3 cameraPosition;
uniform float alphaCutoff;
uniform int textureIsSrgb;
uniform vec3 emission;
uniform int lightCount;
uniform vec3 lightPosition[24];
uniform vec3 lightColor[24];
uniform float lightRadius[24];
out vec4 finalColor;
vec3 toLinear(vec3 c) {
    return mix(c/12.92, pow((c+0.055)/1.055, vec3(2.4)), step(vec3(0.04045), c));
}
vec3 toSrgb(vec3 c) {
    return mix(c*12.92, 1.055*pow(c,vec3(1.0/2.4))-0.055, step(vec3(0.0031308), c));
}
void main() {
    vec4 texel = texture(texture0, uv);
    if (alphaCutoff >= 0.0 && texel.a * colDiffuse.a < alphaCutoff) discard;
    vec3 albedo = (textureIsSrgb == 1 ? toLinear(texel.rgb) : texel.rgb) * colDiffuse.rgb;
    vec3 n = normalize(normal);
    if (!gl_FrontFacing) n = -n;
    float eyeDistance = distance(worldPosition, cameraPosition);
    vec3 light = vec3(0.065,0.075,0.095) + vec3(0.09)*max(0.0,1.0-eyeDistance/5.0);
    for (int i=0; i<lightCount; ++i) {
        vec3 delta=lightPosition[i]-worldPosition;
        float d=length(delta);
        float falloff=max(0.0,1.0-d/lightRadius[i]);
        float diffuse=max(dot(n,delta/max(d,0.001)),0.0);
        light += lightColor[i]*falloff*falloff*(0.12+0.88*diffuse)*1.3;
    }
    vec3 color=toSrgb(max(albedo*light + emission*texel.rgb,vec3(0.0)));
    float fog=smoothstep(28.0,55.0,eyeDistance);
    finalColor=vec4(mix(color,vec3(0.025,0.035,0.05),fog),1.0);
}
