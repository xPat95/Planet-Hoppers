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
out vec4 finalColor;

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
    // Existing recessed ceiling rows and gallery bounce, without global postprocessing.
    lit+=fixture(vec3(-9.0,12.5,-12.0),vec3(0.63,0.77,1.0),0.55,n,v,base);
    lit+=fixture(vec3(9.0,12.5,11.0),vec3(0.72,0.84,1.0),0.55,n,v,base);
    lit+=fixture(vec3(-13.0,6.5,4.0),vec3(1.0,0.79,0.53),0.16,n,v,base);
    lit+=fixture(vec3(0.0,7.5,-28.0),vec3(0.40,0.68,1.0),0.30,n,v,base);
    float fresnel=pow(1.0-max(dot(n,v),0.0),4.0);
    lit+=base*fresnel*vec3(0.04,0.065,0.1)+emissive;
    finalColor=vec4(pow(max(lit,vec3(0.0)),vec3(1.0/2.2)),tex.a*colDiffuse.a);
    if (outline>1.5) finalColor=tex;
    else if (outline>0.5) finalColor=vec4(0.02,0.02,0.03,1.0);
}
