
struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) uv: vec2<f32>, // Add this line
};

@vertex
fn vs_main(model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>((model.position), 1.0);
    out.uv = model.position.xy;
    return out;
}


@group(0) @binding(0)
var read_texture: texture_storage_2d<rgba32float, read>;

struct DisplaySettings {
    sample_type: u32,
}
@group(1) @binding(0)
var<uniform> dis_set: DisplaySettings;

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    var uv = in.uv * 0.5 + 0.5;

    var color: vec3<f32>;

    if (dis_set.sample_type == 0u) {
        color = nns(uv);
    } else if (dis_set.sample_type == 1u) {
        color = bilinear(uv);
    } else {
        color = vec3<f32>(0.0, 0.0, 0.0); // default case
    }

    // i could change this to useing native samplers if i edit the storage texture package to include a texture binging aswell as read/write
    // then have the sampler in display_texture_pipeline


    return vec4(color,  1.0);
}

fn nns(uv: vec2<f32>) -> vec3<f32> {
    let dimensions = textureDimensions(read_texture);

    let uv_nearest = vec2<i32>(floor(uv * vec2<f32>(dimensions.xy)));

    var color = textureLoad(read_texture, uv_nearest).rgb;

    return color;
}

fn bilinear(uv: vec2<f32>) -> vec3<f32> {
    let dimensions = textureDimensions(read_texture);
    let uv_nearest = vec2<i32>(floor(uv * vec2<f32>(dimensions.xy)));

    let one = textureLoad(read_texture, uv_nearest + vec2(-1, -1)).rgb;
    let two = textureLoad(read_texture, uv_nearest + vec2(1, -1)).rgb;
    let three = textureLoad(read_texture, uv_nearest + vec2(-1, 1)).rgb;
    let four = textureLoad(read_texture, uv_nearest + vec2(1, 1)).rgb;

    return (one + two + three + four) / 4.0;
}