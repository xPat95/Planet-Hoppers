#version 330
// Architectural illumination only. Ships keep the existing space shader and key light.
in vec3 worldPosition;
in vec3 normal;
in vec2 uv;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform vec3 lightPosition;
uniform vec3 cameraPosition;
uniform vec3 emissive;
uniform float ambient;
uniform float outline;
uniform float bakedRadiance;
uniform float transportGlow;
out vec4 finalColor;

vec3 transportLight(vec3 color) {
    vec3 delta = worldPosition - vec3(0.0, 1.3, 0.0);
    float reach = clamp(transportGlow * 0.38 / (1.0 + dot(delta, delta) * 0.23), 0.0, 0.8);
    return mix(color, color * vec3(0.58, 0.82, 1.05) + vec3(0.02, 0.10, 0.20), reach);
}

vec3 fixture(vec3 p, vec3 tint, float power, vec3 n, vec3 v, vec3 base) {
    vec3 delta = p-worldPosition;
    float distance2 = dot(delta,delta);
    vec3 l = normalize(delta);
    float diffuse = max(dot(n,l),0.0);
    float highlight = pow(max(dot(n,normalize(l+v)),0.0),56.0);
    return (base*diffuse + vec3(highlight)*0.18) * tint * power/(1.0+distance2*0.009);
}
void main() {
    vec4 tex=texture(texture0,uv);
    // Exported Cycles radiance already contains the approved materials, area-light
    // shadows, indirect illumination, emission and AgX display transform.
    if (bakedRadiance>0.5) {
        // Combined baking cannot retain all view-dependent metallic bounce.
        // A subdued floor reflection keeps upper facets readable without
        // replacing the dominant baked lighting and its contact shadows.
        vec3 n=normalize(normal);
        if (!gl_FrontFacing) n=-n;
        float roof=smoothstep(7.5,11.0,worldPosition.y)*(0.2+0.8*max(-n.y,0.0));
        vec3 bounce=pow(max(colDiffuse.rgb*0.06*roof,vec3(0.0)),vec3(1.0/2.2));
        finalColor=vec4(transportLight(max(tex.rgb,bounce)),tex.a*colDiffuse.a);
        return;
    }
    vec3 base=pow(tex.rgb,vec3(2.2))*colDiffuse.rgb;
    vec3 n=normalize(normal);
    if (!gl_FrontFacing) n=-n;
    vec3 v=normalize(cameraPosition-worldPosition);
    // Gentle brushed machining: subordinate to real cassettes, louvers and bevels.
    float grain=sin(worldPosition.x*73.0+worldPosition.y*17.0+worldPosition.z*43.0);
    float resolved=1.0-smoothstep(0.08,0.4,length(fwidth(worldPosition)));
    base*=1.0+grain*0.018*resolved;
    vec3 lit=base*vec3(0.48,0.59,0.76)*ambient;
    lit+=fixture(lightPosition,vec3(0.83,0.91,1.0),0.65,n,v,base);
    // Approved Blender presentation positions, converted (x,y,z) -> (x,z,-y).
    // Analytic fill approximates the area lights; the platform and ship use other shaders.
    lit+=fixture(vec3(0.0,13.0,11.0),vec3(0.78,0.87,1.0),0.70,n,v,base);
    lit+=fixture(vec3(0.0,13.0,0.0),vec3(0.78,0.87,1.0),0.70,n,v,base);
    lit+=fixture(vec3(0.0,13.0,-11.0),vec3(0.78,0.87,1.0),0.70,n,v,base);
    lit+=fixture(vec3(-13.0,8.0,1.0),vec3(0.78,0.86,1.0),0.55,n,v,base);
    lit+=fixture(vec3(13.0,8.0,1.0),vec3(0.78,0.86,1.0),0.55,n,v,base);
    lit+=fixture(vec3(-13.0,9.0,-11.0),vec3(1.0,0.80,0.56),0.25,n,v,base);
    lit+=fixture(vec3(13.0,9.0,-11.0),vec3(1.0,0.80,0.56),0.25,n,v,base);
    lit+=fixture(vec3(0.0,10.0,-23.0),vec3(0.65,0.78,1.0),0.60,n,v,base);
    float fresnel=pow(1.0-max(dot(n,v),0.0),4.0);
    lit+=base*fresnel*vec3(0.04,0.065,0.1)+emissive;
    finalColor=vec4(transportLight(pow(max(lit,vec3(0.0)),vec3(1.0/2.2))),tex.a*colDiffuse.a);
    if (outline>1.5) finalColor=tex;
    else if (outline>0.5) finalColor=vec4(0.02,0.02,0.03,1.0);
}
