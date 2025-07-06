#version 330
//author https://github.com/autergame

in vec3 UV;

out vec4 FragColor;

uniform samplerCube Skybox;

void main()
{    
    FragColor = texture(Skybox, UV);
}