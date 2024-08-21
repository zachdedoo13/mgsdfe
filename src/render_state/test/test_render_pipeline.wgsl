struct VertexInput {
    @location(0) position: vec3<f32>,
};


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};



@vertex
fn vs_main(
    model: VertexInput,
    @builtin(instance_index) instance_index : u32,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>((model.position), 1.0);

    out.uv = model.position.xy + 0.5;
    return out;
}


@group(0) @binding(0)
var<uniform> time: f32;



// Fragment shader
// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Determine the color based on the 4x4 grid
    var color: vec4<f32>;

    let adjusted_uv = (in.uv + vec2<f32>(0.5, 0.5)) * 0.5 + (sin(time * 1.5) * 0.5 + 0.5);

    let x = adjusted_uv.x * 4.0;
    let y = adjusted_uv.y * 4.0;

    if x < 1.0 {
        if y < 1.0 {
            color = vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red
        } else if y < 2.0 {
            color = vec4<f32>(0.0, 1.0, 0.0, 1.0); // Green
        } else if y < 3.0 {
            color = vec4<f32>(0.0, 0.0, 1.0, 1.0); // Blue
        } else {
            color = vec4<f32>(1.0, 1.0, 0.0, 1.0); // Yellow
        }
    } else if x < 2.0 {
        if y < 1.0 {
            color = vec4<f32>(1.0, 0.5, 0.0, 1.0); // Orange
        } else if y < 2.0 {
            color = vec4<f32>(0.5, 0.0, 0.5, 1.0); // Purple
        } else if y < 3.0 {
            color = vec4<f32>(0.0, 0.5, 0.5, 1.0); // Teal
        } else {
            color = vec4<f32>(0.5, 0.5, 0.5, 1.0); // Gray
        }
    } else if x < 3.0 {
        if y < 1.0 {
            color = vec4<f32>(1.0, 0.0, 0.5, 1.0); // Pink
        } else if y < 2.0 {
            color = vec4<f32>(0.5, 1.0, 0.0, 1.0); // Lime
        } else if y < 3.0 {
            color = vec4<f32>(0.0, 0.5, 1.0, 1.0); // Sky Blue
        } else {
            color = vec4<f32>(1.0, 1.0, 0.5, 1.0); // Light Yellow
        }
    } else {
        if y < 1.0 {
            color = vec4<f32>(0.5, 0.0, 0.0, 1.0); // Dark Red
        } else if y < 2.0 {
            color = vec4<f32>(0.0, 0.5, 0.0, 1.0); // Dark Green
        } else if y < 3.0 {
            color = vec4<f32>(0.0, 0.0, 0.5, 1.0); // Dark Blue
        } else {
            color = vec4<f32>(0.5, 0.5, 0.0, 1.0); // Olive
        }
    }
    return color;
}