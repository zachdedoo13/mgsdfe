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

    float start_eps;
    float max_dist;
    float relaxation;
    float step_scale_factor;
    float eps_scale;

    float pos_x;
    float pos_y;
    float pos_z;

//    vec3 cam_pos;
//    vec3 cam_dir;
} s;

struct Ray { vec3 ro; vec3 rd; };
struct Hit { float d; };


#define FP 200.0
#define MHD 0.001


float sdSphere(vec3 p, float s) {
    return length(p) - s;
}

float sdCube(vec3 p, vec3 b )
{
    vec3 q = abs(p) - b;
    return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0);
}



float sdOctahedronExact(vec3 p, float s)
{
    p = abs(p);
    float m = p.x+p.y+p.z-s;
    vec3 q;
    if( 3.0*p.x < m ) q = p.xyz;
    else if( 3.0*p.y < m ) q = p.yzx;
    else if( 3.0*p.z < m ) q = p.zxy;
    else return m*0.57735027;

    float k = clamp(0.5*(q.z-q.y+s),0.0,s);
    return length(vec3(q.x,q.y-s+k,q.z-k));
}



// methods
vec3 move(vec3 p, vec3 by) {
    return p - by;
}

vec3 rot3D(vec3 p, vec3 rot) {
    // Rotation around X-axis
    float cosX = cos(rot.x);
    float sinX = sin(rot.x);
    mat3 rotX = mat3(
    1.0, 0.0, 0.0,
    0.0, cosX, -sinX,
    0.0, sinX, cosX
    );

    // Rotation around Y-axis
    float cosY = cos(rot.y);
    float sinY = sin(rot.y);
    mat3 rotY = mat3(
    cosY, 0.0, sinY,
    0.0, 1.0, 0.0,
    -sinY, 0.0, cosY
    );

    // Rotation around Z-axis
    float cosZ = cos(rot.z);
    float sinZ = sin(rot.z);
    mat3 rotZ = mat3(
    cosZ, -sinZ, 0.0,
    sinZ, cosZ, 0.0,
    0.0, 0.0, 1.0
    );

    // Apply rotations
    p = rotX * p;
    p = rotY * p;
    p = rotZ * p;

    return p;
}


// unions
Hit opUnion(Hit v1, Hit v2) {
    return v1.d < v2.d ? v1 : v2;
}


Hit map(vec3 p_in) {
    Hit st;
    vec3 t;

    vec3 p = p_in;

    // Existing spheres
    t = move(p, vec3(sin(s.time), 1.0, 0.0));
    st = Hit(sdSphere(t, 0.5));

    t = move(p, vec3(1.0, sin(s.time), 0.0));
    st = opUnion(st, Hit(sdSphere(t, 1.0)));

    t = move(p, vec3(1.0, 1.0, sin(s.time)));
    st = opUnion(st, Hit(sdSphere(t, 0.25)));

    // Spinning squares
    t = move(p, vec3(2.0, 0.0, 0.0));
    t = rot3D(t, vec3(0.0, s.time, 0.0));
    st = opUnion(st, Hit(sdCube(t, vec3(0.5))));

    t = move(p, vec3(0.0, 2.0, 0.0));
    t = rot3D(t, vec3(s.time, 0.0, 0.0));
    st = opUnion(st, Hit(sdCube(t, vec3(0.5))));

    // Spinning octahedrons
    t = move(p, vec3(0.0, 0.0, 2.0));
    t = rot3D(t, vec3(0.0, 0.0, s.time));
    st = opUnion(st, Hit(sdOctahedronExact(t, 0.5)));

    t = move(p, vec3(2.0, 2.0, 2.0));
    t = rot3D(t, vec3(s.time, s.time, s.time));
    st = opUnion(st, Hit(sdOctahedronExact(t, 0.5)));

    return st;
}

Hit CastRay(Ray ray) {
    float t = 0.0;
    for (int i = 0; i < s.steps_per_ray; i++) {
        vec3 p = ray.ro + ray.rd * t;
        Hit hit = map(p);
        t += hit.d;

        if (hit.d < MHD) break;
        if (t > FP) break;
    }
    return Hit(t);
}

//// Adjustable values
//float eps = 0.0001;
//const float maxDist = 16.0;
//float w = 1.05;
//float dw = 1.4;
//float epsScale = 1.05;


Hit CastRayFast(Ray ray) {
    // Adjustable values
    float eps = s.start_eps;
    float maxDist = s.max_dist;
    float w = s.relaxation;
    float dw = s.step_scale_factor;
    float epsScale = s.eps_scale;

    // Initialize variables
    float t = 0.0;
    float prevF = 0.0;
    float prevDt = 0.0;
    bool relaxed = true;

    for (int i = 0; i < s.steps_per_ray; i++) {
        vec3 p = ray.ro + ray.rd * t;
        float f = map(p).d;
        float dt = f * w;

        if (prevF + f < prevDt && relaxed) {
            relaxed = false;
            t += prevDt * (1.0 - w);
            p = ray.ro + ray.rd * t;
            f = map(p).d;
            dt = f * w;
        }

        if (f < eps) {
            return Hit(t);
        } else {
            t += dt;
            prevF = f;
            prevDt = dt;
            if (relaxed) {
                w = mod(fract(w) * dw, 1.0) + 1.0;
            } else {
                w = 1.2;
            }
            eps *= epsScale;
        }

        if (t > maxDist) break;
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
        vec3(s.pos_x, s.pos_y, s.pos_z),
        normalize(vec3(uv, 1.0))
    );


    // path traceing
    Hit test = CastRayFast(ray);

    vec3 col = vec3(test.d * 0.02);

    imageStore(write_tex, gl_uv, vec4(col, 1.0));

}