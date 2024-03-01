// Vertex shader

struct VertexOutput {
    // The @builtin(position) attribute is used to specify the clip-space position of the vertex. This is analogous to GLSL's gl_Position variable.
    // Something to note about @builtin(position), in the fragment shader, this value is in framebuffer space. This means
    // that if your window is 800x600, the x and y of clip_position would be between 0-800 and 0-600 respectively, with
    // the y = 0 being the top of the window. This is different from the clip space, where the x and y are between -1 and 1.
    // This can be useful if you want to know the pixel coordinates of a given fragment, but if you want the position
    // coordinates, you'll have to pass them in separately.
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_pos: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    // Variables defined with ```var``` can be modified but must specify their type. Variables created with ```let```
    //are immutable, but can have their types inferred.
    var output: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    output.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    output.vert_pos = vec2<f32>(x, y);
//    output.vert_pos = output.clip_position.xyz;
    return output;
}


@fragment
// The @location(0) attribute tells WGPU to store the vec4 value returned by this function in the first color target.
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.vert_pos, 0.5, 1.0);
}
