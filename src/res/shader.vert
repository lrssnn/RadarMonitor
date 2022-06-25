#version 150

in vec2 position;
in vec2 texture_pos;

out vec2 Texture_pos;

void main()
{
    Texture_pos = texture_pos;

    gl_Position = vec4(position, 0.0, 1.0);
}

