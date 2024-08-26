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

struct Ray { vec3 ro; vec3 rd; };
struct Hit { float dist; };


#define FP 100.0
#define MHD 0.01



Hit map(vec3 p) {
    return Hit(
        min(
            min(length(p - vec3(sin(s.time) * 2.0)) - 1.0, length(p + vec3(sin(s.time) * 2.0)) - 1.0),
            min(length(p - vec3(sin(s.time) * 2.0, 2.0, 1.0)) - 1.0, length(p + vec3(sin(s.time) * 2.0, -2.0, 1.0)) - 1.0)
        )
    );
}

Hit CastRay(Ray ray) {
    float t = 0.0;
    for (int i = 0; i < s.steps_per_ray; i++) {
        vec3 p = ray.ro + ray.rd * t;
        Hit hit = map(p);
        t += hit.dist;

        if (abs(hit.dist) < MHD) break;
        if (t > FP) Hit(-1.0);
    }
    return Hit(t);
}






void main() {
    ivec2 gl_uv = ivec2(gl_GlobalInvocationID.xy);
    ivec2 dimentions = imageSize(read_tex);
    if (gl_uv.x > dimentions.x || gl_uv.y > dimentions.y) { return; }// bounds check

    float aspect = float(dimentions.x) / float(dimentions.y);

    vec2 uv = vec2(gl_uv.x / float(dimentions.x), gl_uv.y / float(dimentions.y));
    uv = uv * 2.0 - 1.0;
    uv.x *= aspect;

    // setup
    Ray ray = Ray(
        vec3(0.0, 0.0, -8.0),
        vec3(uv, 1.0)
    );


    // path traceing
    Hit test = CastRay(ray);

    vec3 col = vec3(test.dist * 0.02);

    imageStore(write_tex, gl_uv, vec4(col, 1.0));

}