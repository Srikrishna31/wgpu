struct CameraUniform {
    view_proj: mat4x4<f32>,
};

// Because we've created a new bind group, we need to specify which one we're using in the shader. The number is determined
// by out `render_pipeline_layout`. The texture_bind_group_layout is listed first, thus it's group(0), and camera_bind_group
// is second, so it's group(1).
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    // Multiplication order is important when it comes to matrices. The vector goes on the right, and the matrices go on
    // the left in order of importance.
    output.clip_position = camera.view_proj * vec4<f32>(input.pos, 1.0);
    output.tex_coords = input.tex_coords;
    return output;
}

// The variables t_diffuse and s_diffuse are what's known as uniforms.
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(2) @binding(0)
var<uniform> light: Light;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // We don't need (or want) much ambient light, so 0.1 is fine
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    let result = ambient_color * object_color.xyz;

    return vec4<f32>(result, object_color.a);
}