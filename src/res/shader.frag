#version 150

in vec2 Texture_pos;

out vec4 outColor;

uniform sampler2D tex;

void main()
{
    outColor = texture(tex, Texture_pos);
}
