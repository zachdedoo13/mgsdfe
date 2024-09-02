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

struct Ray { vec3 ro; vec3 rd; };
struct Hit { float d;  };


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



// smart if it can hit it checks
#define HITSARRAY vec3[6]
struct Hits { HITSARRAY checks; uint count; };

void sort_aabb_cast(inout Hits hits) {
    for (int i = 1; i < hits.count; i++) {
        vec3 key = hits.checks[i];
        int j = i - 1;

        // Move elements of arr[0..i-1], that are greater than key,
        // to one position ahead of their current position
        while (j >= 0 && hits.checks[j].x > key.x) {
            hits.checks[j + 1] = hits.checks[j];
            j = j - 1;
        }
        hits.checks[j + 1] = key;
    }
}

Hits aabb_smart_cast(Ray ray) {
    Hits hits;
    hits.count = 0;

    vec3 t1tr = vec3(0.0, 0.0, 3.0);
    if (bool_hit(intersectAABB(ray, from_pos_size(t1tr, vec3(1.5))))) {
        vec2 c0 = intersectAABB(ray, from_pos_size(t1tr + vec3(0.0), vec3(0.5)));
        if (bool_hit(c0)) { hits.checks[hits.count] = vec3(c0, 0.0); hits.count += 1; }  // s0

        vec2 c1 = intersectAABB(ray, from_pos_size(t1tr + vec3(1.0, 1.0, 0.0), vec3(0.5)));
        if (bool_hit(c1)) { hits.checks[hits.count] = vec3(c1, 1.0); hits.count += 1; }  // s1
    }

    vec3 t2tr = vec3(1.0, -1.0, 2.0);
    if (bool_hit(intersectAABB(ray, from_pos_size(t2tr, vec3(1.5))))) {
        vec2 c2 = intersectAABB(ray, from_pos_size(t2tr + vec3(0.0), vec3(0.5)));
        if (bool_hit(c2)) { hits.checks[hits.count] = vec3(c2, 2.0); hits.count += 1; }  // s2

        vec2 c3 = intersectAABB(ray, from_pos_size(t2tr + vec3(1.0, 1.0, 0.0), vec3(0.5)));
        if (bool_hit(c3)) { hits.checks[hits.count] = vec3(c3, 3.0); hits.count += 1; }  // s3
    }

    vec3 t3tr = vec3(-1.0, 0.0, 1.0);
    if (bool_hit(intersectAABB(ray, from_pos_size(t3tr, vec3(1.5))))) {
        vec2 c4 = intersectAABB(ray, from_pos_size(t3tr + vec3(0.0), vec3(0.5)));
        if (bool_hit(c4)) { hits.checks[hits.count] = vec3(c4, 4.0); hits.count += 1; }  // s4

        vec2 c5 = intersectAABB(ray, from_pos_size(t3tr + vec3(1.0, 1.0, 0.0), vec3(0.5)));
        if (bool_hit(c5)) { hits.checks[hits.count] = vec3(c5, 5.0); hits.count += 1; }  // s5
    }

    sort_aabb_cast(hits);

    return hits;
}

Hit map_smart_aabb(vec3 p_in, float f_index) {
    Hit back = Hit(100000.0);
    vec3 t = p_in;

    uint index = uint(f_index);
    switch (index) {
        case 0:
        {
            Hit u1 = back;
            vec3 u1t = move(t, vec3(0.0, 0.0, 3.0));
            {
                vec3 u1s1t = u1t;
                u1s1t = move(u1s1t, vec3(0.0));
                Hit u1s1 = Hit(sdCube(u1s1t, vec3(0.5)));
                u1 = opUnion(u1, u1s1);
            }
            back = opUnion(back, u1);
        }
        break;

        case 1:
        {
            Hit u1 = back;
            vec3 u1t = move(t, vec3(0.0, 0.0, 3.0));
            {
                vec3 u1s2t = u1t;
                u1s2t = move(u1s2t, vec3(1.0, 1.0, 0.0));
                Hit u1s2 = Hit(sdSphere(u1s2t, 0.5));
                u1 = opUnion(u1, u1s2);
            }
            back = opUnion(back, u1);
        }
        break;

        case 2:
        {
            Hit u2 = back;
            vec3 u2t = move(t, vec3(1.0, -1.0, 2.0));
            {
                vec3 u2s1t = u2t;
                u2s1t = move(u2s1t, vec3(0.0));
                Hit u2s1 = Hit(sdCube(u2s1t, vec3(0.5)));
                u2 = opUnion(u2, u2s1);
            }
            back = opUnion(back, u2);
        }
        break;

        case 3:
        {
            Hit u2 = back;
            vec3 u2t = move(t, vec3(1.0, -1.0, 2.0));
            {
                vec3 u2s2t = u2t;
                u2s2t = move(u2s2t, vec3(1.0, 1.0, 0.0));
                float scale = 1.0 / 0.5;
                Hit u2s2 = Hit(sdMandelbulb(u2s2t * scale, MBS) / scale);
                u2 = opUnion(u2, u2s2);
            }
            back = opUnion(back, u2);
        }
        break;

        case 4:
        {
            Hit u3 = back;
            vec3 u3t = move(t, vec3(-1.0, 0.0, 1.0));
            {
                vec3 u3s1t = u3t;
                u3s1t = move(u3s1t, vec3(0.0));
                float scale = 1.0 / 0.5;
                Hit u3s1 = Hit(sdMandelbulb(u3s1t * scale, MBS) / scale);
                u3 = opUnion(u3, u3s1);
            }
            back = opUnion(back, u3);
        }
        break;

        case 5:
        {
            Hit u3 = back;
            vec3 u3t = move(t, vec3(-1.0, 0.0, 1.0));
            {
                vec3 u3s2t = u3t;
                u3s2t = move(u3s2t, vec3(1.0, 1.0, 0.0));
                float scale = 1.0 / 0.5;
                Hit u3s2 = Hit(sdMandelbulb(u3s2t * scale, MBS) / scale);
                u3 = opUnion(u3, u3s2);
            }
            back = opUnion(back, u3);
        }
        break;

        default:
        break;
    }

    return back;
}

float disclose_index(vec3 data, inout int steps, Ray ray) {
    float t = data.x;
    for (int i = 0; i < s.steps_per_ray; i++) {
        vec3 p = ray.ro + ray.rd * t;
        Hit hit = map_smart_aabb(p, data.z);
        t += hit.d;

        if (hit.d < MHD) break;
        if (t > FP) break;
        if (t > data.y) break;
    }

    return t;
}

vec4 cast_smart_aabb(Ray ray) {
    Hits hits = aabb_smart_cast(ray);
    float t = 100000.0;
    int steps = 0;
    uint i = 0;
    while (i < hits.count) {
        float disclose = disclose_index(hits.checks[i], steps, ray);

        // if hit in the aabb
        if (disclose < hits.checks[i].y) {
            // overlap was invalid
            if (disclose < t) { t = disclose; }

            // overlap detected
            if (hits.checks[i].y < hits.checks[i + 1].x) {

            }
            // no overlap and hit object
            else {
                break;
            }
        }

        i += 1;
    }

    return vec4(t * 0.2, vec2(0.0, 1.0), 1.0);
}



// bitwise if it can hit it checks
#define BITCHECKS uint[1]



// if it can hit it checks
#define CHECKS bool[6]

CHECKS aabb_simple_array_bounds_checker(Ray ray) {
    CHECKS checks = CHECKS(false, false, false, false, false, false);

    // transform 1
    vec3 t1tr = vec3(0.0, 0.0, 3.0);
    if (bool_hit(intersectAABB(ray, from_pos_size(t1tr, vec3(1.5))))) {
        checks[0] = (bool_hit(intersectAABB(ray, from_pos_size(t1tr + vec3(0.0), vec3(0.5)))));  //s1
        checks[1] = (bool_hit(intersectAABB(ray, from_pos_size(t1tr + vec3(1.0, 1.0, 0.0), vec3(0.5)))));   //s2
    }

    // transform 2
    vec3 t2tr = vec3(1.0, -1.0, 2.0);
    if (bool_hit(intersectAABB(ray, from_pos_size(t2tr, vec3(2.0))))) {
        checks[2] = (bool_hit(intersectAABB(ray, from_pos_size(t2tr + vec3(0.0), vec3(0.5)))));  //s3
        checks[3] = (bool_hit(intersectAABB(ray, from_pos_size(t2tr + vec3(1.0, 1.0, 0.0), vec3(0.5)))));   //s4
    }

    // transform 3
    vec3 t3tr = vec3(-1.0, 0.0, 1.0);
    if (bool_hit(intersectAABB(ray, from_pos_size(t3tr, vec3(2.0))))) {
        checks[4] = (bool_hit(intersectAABB(ray, from_pos_size(t3tr + vec3(0.0), vec3(0.5)))));  //s5
        checks[5] = (bool_hit(intersectAABB(ray, from_pos_size(t3tr + vec3(1.0, 1.0, 0.0), vec3(0.5)))));   //s6
    }

    return checks;
}

float aabb_simple_array_debug(Ray ray) {
    int count = 0;

    // Transform 1
    vec3 t1tr = vec3(0.0, 0.0, 3.0);
    if (bool_hit(intersectAABB(ray, from_pos_size(t1tr, vec3(1.5))))) {
        count += 1;
        if (bool_hit(intersectAABB(ray, from_pos_size(t1tr + vec3(0.0), vec3(0.5))))) {
            count += 1;
        }  // s1
        if (bool_hit(intersectAABB(ray, from_pos_size(t1tr + vec3(1.0, 1.0, 0.0), vec3(0.5))))) {
            count += 1;
        }  // s2
    }

    // Transform 2
    vec3 t2tr = vec3(1.0, -1.0, 2.0);
    if (bool_hit(intersectAABB(ray, from_pos_size(t2tr, vec3(2.0))))) {
//        count += 1;
        if (bool_hit(intersectAABB(ray, from_pos_size(t2tr + vec3(0.0), vec3(0.5))))) {
            count += 1;
        }  // s3
        if (bool_hit(intersectAABB(ray, from_pos_size(t2tr + vec3(1.0, 1.0, 0.0), vec3(0.5))))) {
            count += 1;
        }  // s4
    }

    // Transform 3
    vec3 t3tr = vec3(-1.0, 0.0, 1.0);
    if (bool_hit(intersectAABB(ray, from_pos_size(t3tr, vec3(2.0))))) {
//        count += 1;
        if (bool_hit(intersectAABB(ray, from_pos_size(t3tr + vec3(0.0), vec3(0.5))))) {
            count += 1;
        }  // s5
        if (bool_hit(intersectAABB(ray, from_pos_size(t3tr + vec3(1.0, 1.0, 0.0), vec3(0.5))))) {
            count += 1;
        }  // s6
    }

    return float(count) / 9.0;
}

Hit map_simple_array_bounds_checker(vec3 p_in, CHECKS checks) {
    Hit back = Hit(100000.0);
    vec3 t = p_in;

    // Transform 1
    if (checks[0] || checks[1]) {
        Hit u1 = back;
        vec3 u1t = move(t, vec3(0.0, 0.0, 3.0));
        {
            if (checks[0]) {
                vec3 u1s1t = u1t;
                u1s1t = move(u1s1t, vec3(0.0));

                Hit u1s1 = Hit(sdCube(u1s1t, vec3(0.5)));
                u1 = opUnion(u1, u1s1);
            }

            if (checks[1]) {
                vec3 u1s2t = u1t;
                u1s2t = move(u1s2t, vec3(1.0, 1.0, 0.0));

                Hit u1s2 = Hit(sdSphere(u1s2t, 0.5));
                u1 = opUnion(u1, u1s2);
            }
        }

        back = opUnion(back, u1);
    }

    // Transform 2
    if (checks[2] || checks[3]) {
        Hit u2 = back;
        vec3 u2t = move(t, vec3(1.0, -1.0, 2.0));
        {
            if (checks[2]) {
                vec3 u2s1t = u2t;
                u2s1t = move(u2s1t, vec3(0.0));

                Hit u2s1 = Hit(sdCube(u2s1t, vec3(0.5)));
                u2 = opUnion(u2, u2s1);
            }

            if (checks[3]) {
                vec3 u2s2t = u2t;
                u2s2t = move(u2s2t, vec3(1.0, 1.0, 0.0));

                float scale = 1.0 / 0.5;
                Hit u2s2 = Hit(sdMandelbulb(u2s2t * scale, MBS) / scale);
                u2 = opUnion(u2, u2s2);
            }
        }

        back = opUnion(back, u2);
    }

    // Transform 3
    if (checks[4] || checks[5]) {
        Hit u3 = back;
        vec3 u3t = move(t, vec3(-1.0, 0.0, 1.0));
        {
            if (checks[4]) {
                vec3 u3s1t = u3t;
                u3s1t = move(u3s1t, vec3(0.0));

                float scale = 1.0 / 0.5;
                Hit u3s1 = Hit(sdMandelbulb(u3s1t * scale, MBS) / scale);
                u3 = opUnion(u3, u3s1);
            }

            if (checks[5]) {
                vec3 u3s2t = u3t;
                u3s2t = move(u3s2t, vec3(1.0, 1.0, 0.0));

                float scale = 1.0 / 0.5;
                Hit u3s2 = Hit(sdMandelbulb(u3s2t * scale, MBS) / scale);
                u3 = opUnion(u3, u3s2);
            }
        }

        back = opUnion(back, u3);
    }

    return back;
}

vec4 cast_simple_array_bounds_checker(Ray ray) {
    CHECKS checks = aabb_simple_array_bounds_checker(ray);

    float t = 0.0;
    for (int i = 0; i < s.steps_per_ray; i++) {
        vec3 p = ray.ro + ray.rd * t;
        Hit hit = map_simple_array_bounds_checker(p, checks);
        t += hit.d;

        if (hit.d < MHD) break;
        if (t > FP) break;
    }

//    float debug_col = aabb_simple_array_debug(ray);
    float debug_col = 0.0;

    return vec4(t * 0.2, vec2(debug_col), 1.0);
}



// kinda shit
Hit map_simple_sdf_bounds(vec3 p_in) {
    Hit back = Hit(100000.0);
    vec3 t = p_in;

    // Transform 1
    {
        Hit u1 = back;
        vec3 u1t = move(t, vec3(0.0, 0.0, 3.0));
        if (sdSphere(u1t, 2.5) < 0.1) {
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

        if (sdSphere(u2t, 2.5) < 0.1) {
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
                sdMandelbulb(u2s2t * scale, MBS) / scale
                );
                u2 = opUnion(u2, u2s2);
            }
        }

        back = opUnion(back, u2);
    }

    // Transform 3
    {
        Hit u3 = back;
        vec3 u3t = move(t, vec3(-1.0, 0.0, 1.0));

        if (sdSphere(u3t, 2.5) < 0.1) {
            {
                vec3 u3s1t = u3t;
                u3s1t = move(u3s1t, vec3(0.0));

                float scale = 1.0 / 0.5;
                Hit u3s1 = Hit(
                sdMandelbulb(u3s1t * scale, MBS ) / scale
                );
                u3 = opUnion(u3, u3s1);
            }

            {
                vec3 u3s2t = u3t;
                u3s2t = move(u3s2t, vec3(1.0, 1.0, 0.0));

                float scale = 1.0 / 0.5;
                Hit u3s2 = Hit(
                sdMandelbulb(u3s2t * scale, MBS) / scale
                );
                u3 = opUnion(u3, u3s2);
            }
        }

        back = opUnion(back, u3);
    }

    return back;
}

Hit cast_simple_sdf_bounds(Ray ray) {
    float t = 0.0;
    for (int i = 0; i < s.steps_per_ray; i++) {
        vec3 p = ray.ro + ray.rd * t;
        Hit hit = map_simple_sdf_bounds(p);
        t += hit.d;

        if (hit.d < MHD) break;
        if (t > FP) break;
    }
    return Hit(t);
}


// 350-380
Hit map_brute_force(vec3 p_in) {
    Hit back = Hit(100000.0);
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
//                vec3 scale = vec3(1.0, 1.0, 10.0) * 2.0;
//                float scale = 2.0 * 2.0;

                vec3 u1s1t = u1t;
//                u1s1t /= scale;
                u1s1t = move(u1s1t, vec3(0.0));

                Hit u1s1 = Hit(sdCube(u1s1t, vec3(0.5)));
//                u1s1.d *= scale;
//                u1s1.d *= min(min(scale.x, scale.y), scale.z);



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
                    sdMandelbulb(u2s2t * scale, MBS) / scale
                );
                u2 = opUnion(u2, u2s2);
            }
        }

        back = opUnion(back, u2);
    }

    // Transform 3
    {
        Hit u3 = back;
        vec3 u3t = move(t, vec3(-1.0, 0.0, 1.0));
        {
            {
                vec3 u3s1t = u3t;
                u3s1t = move(u3s1t, vec3(0.0));

                float scale = 1.0 / 0.5;
                Hit u3s1 = Hit(
                sdMandelbulb(u3s1t * scale, MBS) / scale
                );
                u3 = opUnion(u3, u3s1);
            }

            {
                vec3 u3s2t = u3t;
                u3s2t = move(u3s2t, vec3(1.0, 1.0, 0.0));

                float scale = 1.0 / 0.5;
                Hit u3s2 = Hit(
                sdMandelbulb(u3s2t * scale, MBS) / scale
                );
                u3 = opUnion(u3, u3s2);
            }
        }

        back = opUnion(back, u3);
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

// tex
Hit map_tex_test(vec3 p_in) {
    Hit back = Hit(100000.0);
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

    return back;
}

Hit cast_tex_test(Ray ray) {
    float t = 0.0;
    for (int i = 0; i < s.steps_per_ray; i++) {
        vec3 p = ray.ro + ray.rd * t;
        Hit hit = map_tex_test(p);
        t += hit.d;

        if (hit.d < MHD) break;
        if (t > FP) break;
    }
    return Hit(t);
}


////////////////////
/// Pathtraceing ///
////////////////////
vec4 pathtrace(Ray ray) {
    vec4 back;
    Hit test;

    switch (s.mode) {
        case -1:
            test = cast_tex_test(ray);
            back = vec4(vec2(test.d * 0.2), 1.0, 0.4);
            break;
        case 0:
            test = cast_ray_brute_force(ray);
            back = vec4(vec2(test.d * 0.02), 0.0, 1.0);
            break;

        case 1:
            test = cast_simple_sdf_bounds(ray);
            back = vec4(vec2(test.d * 0.2), 1.0, 1.0);
            break;

        case 2:
            back = cast_simple_array_bounds_checker(ray);
            break;

        case 3:
            back = cast_smart_aabb(ray);
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

    // Usage
    ray.rd = rotateRayDirection(ray.rd, vec3(s.rot_x, s.rot_y, s.rot_z));

    // path traceing

    vec4 trace = pathtrace(ray);

    imageStore(write_tex, gl_uv, trace);

}







