#version 450

layout (local_size_x = 16, local_size_y = 16, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba32f) readonly uniform image2D read_tex;
layout(set = 1, binding = 0, rgba32f) writeonly uniform image2D write_tex;

layout(set = 2, binding = 0) uniform PathTracerUniformSettings {
    float time;
    int frame;

    int last_clear_frame;
    int samples_per_frame;
    int steps_per_ray;

    int bounces;
    float fov;
} s;


//Hit CastRay(Ray ray) {
//    float t = 0.0;
//    Mat mat;
//    for (int i = 0; i < STEPS; i++) {
//        vec3 p = ray.ro + ray.rd * t;
//        Hit hit = map(p);
//        mat = hit.mat;
//        t += hit.d;
//
//        if (abs(hit.d) < MHD) break;
//        if (t > FP) break;
//    }
//    return Hit(t, mat);
//}






void main() {
    ivec2 gl_uv = ivec2(gl_GlobalInvocationID.xy);
    ivec2 dimentions = imageSize(read_tex);
    if (gl_uv.x > dimentions.x || gl_uv.y > dimentions.y) { return; } // bounds check

    vec2 uv = vec2(gl_uv.x / float(dimentions.x), gl_uv.y / float(dimentions.y));
    uv = uv * 2.0 - 1.0;


    // path traceing



    vec3 col = imageLoad(read_tex, gl_uv).rgb;


    imageStore(write_tex, gl_uv, vec4(uv, abs(sin(s.time)), 1.0));

}