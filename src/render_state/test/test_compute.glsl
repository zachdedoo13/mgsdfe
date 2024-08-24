#version 450

layout (local_size_x = 16, local_size_y = 16, local_size_z = 1) in;
layout(set = 0, binding = 0, rgba32f) uniform image2D render_texture;




void main() {
    ivec2 gl_uv = ivec2(gl_GlobalInvocationID.xy);
    ivec2 dimentions = imageSize(render_texture);
    if (gl_uv.x > dimentions.x || gl_uv.y > dimentions.y) { return; } // bounds check

    vec2 uv = vec2(gl_uv.x / float(dimentions.x), gl_uv.y / float(dimentions.y));
    uv = uv * 2.0 - 1.0;
//    uv.x *= c.aspect;



    imageStore(render_texture, gl_uv, vec4(uv, 1.0, 1.0));
}