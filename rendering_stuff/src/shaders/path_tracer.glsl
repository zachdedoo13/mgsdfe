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
} s;

struct Ray { vec3 ro; vec3 rd; };
struct Hit { float d; };


#define FP 200.0
#define MHD 0.001

//////////////
/// SHAPES ///
//////////////

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

float sdMandelbulb(vec3 pos, float Power) {
    vec3 z = pos;
    float dr = 1.0;
    float r = 0.0;
    const int MAX_FRACTAL_ITERATIONS = 100; // Define the maximum number of iterations

    for (int i = 0; i < MAX_FRACTAL_ITERATIONS; ++i) {
        r = length(z);
        if (r > 2.0) break;

        float theta = acos(z.z / r);
        float phi = atan(z.y, z.x);
        dr = pow(r, Power - 1.0) * Power * dr + 1.0;
        float zr = pow(r, Power);
        theta *= Power;
        phi *= Power;

        z = zr * vec3(
        sin(theta) * cos(phi),
        sin(phi) * sin(theta),
        cos(theta)
        );
        z += pos;
    }

    return 0.5 * log(r) * r / dr;
}


///////////////
/// Methods ///
///////////////

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


//////////////
/// Unions ///
//////////////

Hit opUnion(Hit v1, Hit v2) {
    return v1.d < v2.d ? v1 : v2;
}



///////////////////
/// Raymarching ///
///////////////////

// test

// Transform 1 @(0.0, 0.0, 3.0) B=2.0x3
    // cube 0.5x3 @(0.0, 0.0, 0.0)
    // curcle 0.5 @(1.0, 1.0, 0.0)

// Transform 2 @(1.0, -1.0, 2.0) B=2.0x3
    // cube 0.5x3 @(0.0, 0.0, 0.0) lone box
    // mandlebulb 0.5 scale = 0.5 @(1.0, 1.0, 0.0) lone box pow (8.0 + sin(s.time * 0.6) * 4.0)





Hit map_brute_force(vec3 p_in) {
    Hit back = Hit(100000.0);
    vec3 t = p_in;

    // Transform 1
    {
        Hit u1 = back;
        vec3 u1t = move(t, vec3(0.0, 0.0, 3.0));
        {
            {
                vec3 u1s1t = u1t;
                u1s1t = move(u1s1t, vec3(0.0));

                Hit u1s1 = Hit(sdCube(u1s1t, vec3(0.5)));
                u1 = opUnion(u1, u1s1);
            }

            {
                vec3 u1s2t = u1t;
                u1s2t = move(u1s2t, vec3(1.0, 1.0, 0.0));

                Hit u1s2 = Hit(sdSphere(u1s2t, 0.5));
                u1 = opUnion(u1, u1s2);
            }
        }

        back = opUnion(back, u1);
    }

    // Transform 2
    {
        Hit u2 = back;
        vec3 u2t = move(t, vec3(1.0, -1.0, 2.0));
        {
            {
                vec3 u2s1t = u2t;
                u2s1t = move(u2s1t, vec3(0.0));

                Hit u2s1 = Hit(sdCube(u2s1t, vec3(0.5)));
                u2 = opUnion(u2, u2s1);
            }

            {
                vec3 u2s2t = u2t;
                u2s2t = move(u2s2t, vec3(1.0, 1.0, 0.0));

                float scale = 1.0 / 0.5;
                Hit u2s2 = Hit(
                    sdMandelbulb(u2s2t * scale, 8.0 + sin(s.time * 0.6) * 4.0 ) / scale
                );
                u2 = opUnion(u2, u2s2);
            }
        }

        back = opUnion(back, u2);
    }

    return back;
}

Hit cast_ray_brute_force(Ray ray) {
    float t = 0.0;
    for (int i = 0; i < s.steps_per_ray; i++) {
        vec3 p = ray.ro + ray.rd * t;
        Hit hit = map_brute_force(p);
        t += hit.d;

        if (hit.d < MHD) break;
        if (t > FP) break;
    }
    return Hit(t);
}


////////////////////
/// Pathtraceing ///
////////////////////
const int mode = 0;

vec4 pathtrace(Ray ray) {
    vec4 back;

    switch (mode) {
        case 0:
            Hit test = cast_ray_brute_force(ray);
            back = vec4(vec3(test.d * 0.2), 1.0);
            break;

        case 1:
            // Code for case 1
            break;

        case 2:
            // Code for case 2
            break;

        default:
            // Code for default case
            break;
    }


    return back;
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

    vec4 trace = pathtrace(ray);

    imageStore(write_tex, gl_uv, trace);

}