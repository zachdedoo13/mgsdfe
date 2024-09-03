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

    int mode;
    float max_dist;
    float relaxation;
    float step_scale_factor;
    float eps_scale;

    float pos_x;
    float pos_y;
    float pos_z;

    float rot_x;
    float rot_y;
    float rot_z;
} s;

layout(set = 3, binding = 0) uniform texture2D myTexture;
layout(set = 3, binding = 1) uniform sampler mySampler;


struct Mat { vec3 col; };
#define MDEF Mat(vec3(0.0))

struct Ray { vec3 ro; vec3 rd; };
struct Hit { float d; Mat mat; };


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

vec3 rotateRayDirection(vec3 direction, vec3 rotation) {
    // Rotation around X-axis
    float cosX = cos(rotation.x);
    float sinX = sin(rotation.x);
    mat3 rotX = mat3(
    1.0, 0.0, 0.0,
    0.0, cosX, -sinX,
    0.0, sinX, cosX
    );

    // Rotation around Y-axis
    float cosY = cos(rotation.y);
    float sinY = sin(rotation.y);
    mat3 rotY = mat3(
    cosY, 0.0, sinY,
    0.0, 1.0, 0.0,
    -sinY, 0.0, cosY
    );

    // Rotation around Z-axis
    float cosZ = cos(rotation.z);
    float sinZ = sin(rotation.z);
    mat3 rotZ = mat3(
    cosZ, -sinZ, 0.0,
    sinZ, cosZ, 0.0,
    0.0, 0.0, 1.0
    );

    // Apply rotations
    direction = rotX * direction;
    direction = rotY * direction;
    direction = rotZ * direction;

    return direction;
}

float scale_correction(float d, float s) {
    //                u1s1.d *= min(min(scale.x, scale.y), scale.z);
    //    return d * min(min(s.x, s.y), s.z);
    return d * s;
}


//////////////
/// Unions ///
//////////////

Hit opUnion(Hit v1, Hit v2) {
    return v1.d < v2.d ? v1 : v2;
}

////////////
/// AABB ///
////////////

struct AABB {
    vec3 min;
    vec3 max;
};

AABB from_pos_size(vec3 pos, vec3 size) {
    AABB cube;
    cube.min = pos - size;
    cube.max = pos + size;

    return cube;
}

vec2 intersectAABB(Ray ray, AABB cube) {
    vec3 tMin = (cube.min - ray.ro) / ray.rd;
    vec3 tMax = (cube.max - ray.ro) / ray.rd;
    vec3 t1 = min(tMin, tMax);
    vec3 t2 = max(tMin, tMax);
    float tNear = max(max(t1.x, t1.y), t1.z);
    float tFar = min(min(t2.x, t2.y), t2.z);
    return vec2(tNear, tFar);
}

bool bool_hit(vec2 intersect) {
    return intersect.x < intersect.y && intersect.y > 0.0;
}


////////////////////////
/// Bit manipulation ///
////////////////////////

// currently fucks shit up, idk why
void setBoolAtIndex(inout uint data, int index, bool value) {
    if (value) {
        data |= (1u << index);  // Set the bit at 'index' to 1
    } else {
        data &= ~(1u << index); // Set the bit at 'index' to 0
    }
}

bool getBoolAtIndex(uint data, int index) {
    return (data & (1u << index)) != 0u; // Check if the bit at 'index' is 1
}

///////////////////
/// Raymarching ///
///////////////////
#define MBS 8.0

// test

// Transform 1 @(0.0, 0.0, 3.0) B=2.0x3
// cube 0.5x3 @(0.0, 0.0, 0.0)   //s1
// sphere 0.5 @(1.0, 1.0, 0.0)   //s2

// Transform 2 @(1.0, -1.0, 2.0) B=2.0x3
// cube 0.5x3 @(0.0, 0.0, 0.0) lone box   //s3
// mandelbulb 0.5 scale = 0.5 @(1.0, 1.0, 0.0) lone box pow (8.0 + sin(s.time * 0.6) * 4.0)   //s4

// Transform 3 @(-1.0, 0.0, 1.0) B=2.0x3
// mandelbulb 0.5 scale = 0.5 @(0.0, 0.0, 0.0) pow (8.0 + sin(s.time * 0.6) * 4.0)   //s5
// mandelbulb 0.5 scale = 0.5 @(1.0, 1.0, 0.0) pow (8.0 + sin(s.time * 0.6) * 4.0)   //s6

// mandlebulbs have a size of 2 unless scaled down
// tex

Hit map_tex_test(vec3 p_in) {
    Hit back = Hit(100000.0, MDEF);
    vec3 t = p_in;

    // Transform 1
    {
        Hit u1 = back;

        float scale = 1.0;

        vec3 u1t = t;
        u1t /= scale;
        u1t = move(u1t, vec3(0.0, 0.0, 3.0));
        {
            {
                vec3 u1s1t = u1t;
                u1s1t = move(u1s1t, vec3(0.0));

                Hit u1s1 = Hit(sdCube(u1s1t, vec3(0.5)), Mat(vec3(0.2, 1.0, 0.6)));

                u1 = opUnion(u1, u1s1);
            }

            {
                vec3 u1s2t = u1t;
                u1s2t = move(u1s2t, vec3(1.0, 1.0, 0.0));

                Hit u1s2 = Hit(sdSphere(u1s2t, 0.5), Mat(vec3(1.0)));
                u1 = opUnion(u1, u1s2);
            }
        }

        back = opUnion(back, u1);
    }

    return back;
}

Hit cast_tex_test(Ray ray) {
    float t = 0.0;
    Hit hit;
    for (int i = 0; i < s.steps_per_ray; i++) {
        vec3 p = ray.ro + ray.rd * t;
        hit = map_tex_test(p);
        t += hit.d;

        if (hit.d < MHD) { return Hit(t, hit.mat); };
        if (t > FP) break;
    }
    return Hit(t, MDEF);
}


////////////////////
/// Pathtraceing ///
////////////////////
vec4 pathtrace(Ray ray) {
    vec4 back;
    Hit test;

    test = cast_tex_test(ray);
    back = vec4(test.mat.col, 1.0);


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

    // Usage
    ray.rd = rotateRayDirection(ray.rd, vec3(s.rot_x, s.rot_y, s.rot_z));

    // path traceing

    vec4 trace = pathtrace(ray);

    // Sample from the texture at mipmap level 2
    float mipLevel = 1.0;
    vec4 color = textureLod(sampler2D(myTexture, mySampler), uv * 0.5 + 0.5, mipLevel);


    imageStore(write_tex, gl_uv, color + trace);
}

