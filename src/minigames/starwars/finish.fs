#version 330
in vec2 fragTexCoord;
in vec4 fragColor;
uniform sampler2D texture0;
out vec4 finalColor;
void main(){
 vec2 uv=fragTexCoord;vec3 c=texture(texture0,uv).rgb;
 vec3 glow=vec3(0.0);
 for(int x=-2;x<=2;x++)for(int y=-2;y<=2;y++){
 vec3 s=texture(texture0,uv+vec2(x,y)*vec2(1.0/1280.0,1.0/800.0)*2.2).rgb;
 float l=max(s.r,max(s.g,s.b));glow+=s*max(l-0.62,0.0)*0.065;
 }
 c+=glow;
 vec2 p=uv*2.0-1.0;float vignette=1.0-0.18*dot(p,p);
 c=pow(c,vec3(0.94))*vignette;
 finalColor=vec4(c,1.0);
}
