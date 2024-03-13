const PI : f32 = 3.1415926535897932384626433832795;

// Each face of the cube map has an orientation to it, so we need to store it.
struct Face {
    forward: vec3<f32>,
    up: vec3<f32>,
    right: vec3<f32>,
}

// The equirectangular src texture
@group(0)
@binding(0)
var src: texture_2d<f32>;

// The dst cube map texture
// While `dst` is a cube texture, it's stored as an array of 2d textures
// The type of binding we're using here is a storage texture. An array storage texture, to be precise. This is a unique
// binding only available to compute shaders. It allows us to write directly to the texture.
// When using a storage texture binding, we need to specify the format of the texture. If you try to bind a texture
// with a different format, wgpu will panic.
@group(0)
@binding(1)
var dst: texture_storage_2d_array<rgba32float, write>;

// The `workgroup_size` decorator tells the dimensions of the workgroup's local grid of invocations. Because we are
// dispatching one workgroup for every pixel in the texture, we have each workgroup be a 16x16x1 grid. This means
// each workgroup can have 256 threads(invocations) to work with.
@compute
@workgroup_size(16, 16, 1)
fn compute_direct_to_cubemap(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    // If texture size is not divisible by 32, we need to make sure we don't try to write to pixels that don't exist.
    if global_id.x >= u32(textureDimensions(dst).x) {
        return;
    }

    var FACES : array<Face, 6> = array(
        // FACES +X
        Face(
            vec3<f32>(1.0, 0.0, 0.0),  // forward
            vec3<f32>(0.0, 1.0, 0.0),  // up
            vec3<f32>(0.0, 0.0, -1.0), // right
        ),
        // FACES -X
        Face(
            vec3<f32>(-1.0, 0.0, 0.0), // forward
            vec3<f32>(0.0, 1.0, 0.0),  // up
            vec3<f32>(0.0, 0.0, 1.0),  // left
        ),
        // FACES +Y
        Face(
            vec3<f32>(0.0, -1.0, 0.0),  // forward
            vec3<f32>(0.0, 0.0, 1.0), // up
            vec3<f32>(1.0, 0.0, 0.0),  // right
        ),
        // FACES -Y
        Face(
            vec3<f32>(0.0, 1.0, 0.0),  // forward
            vec3<f32>(0.0, 0.0, -1.0), // up
            vec3<f32>(1.0, 0.0, 0.0),  // right
        ),
        // FACES +Z
        Face(
            vec3<f32>(0.0, 0.0, 1.0),  // forward
            vec3<f32>(0.0, 1.0, 0.0),  // up
            vec3<f32>(1.0, 0.0, 0.0),  // right
        ),
        // FACES -Z
        Face(
            vec3<f32>(0.0, 0.0, -1.0), // forward
            vec3<f32>(0.0, 1.0, 0.0),  // up
            vec3<f32>(-1.0, 0.0, 0.0), // right
        ),
    );

    // Get texture coords relative to cubemap face
    let dst_dimensions = vec2<f32>(textureDimensions(dst));
    let cube_uv = vec2<f32>(global_id.xy) / dst_dimensions * 2.0 - 1.0;

    // Get spherical coordinate from cube_uv
    let face = FACES[global_id.z];
    let spherical = normalize(face.forward + face.right * cube_uv.x + face.up * cube_uv.y);

    // Get coordinate on the equirectangular src texture
    let inv_atan = vec2(0.1591, 0.3183);
    let eq_uv = vec2(atan2(spherical.z, spherical.x) , asin(spherical.y)) * inv_atan) + 0.5;
    let eq_pixel = vec2<i32>(eq_uv * vec2<f32>(textureDimensions(src)));

    // We use textureLoad() as textureSample() is not allowed in compute shaders
    var sample = textureLoad(src, eq_pixel, 0);

    textureStore(dst, global_id.xy, global_id.z, sample);
}