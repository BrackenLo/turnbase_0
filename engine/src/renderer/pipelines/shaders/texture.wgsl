//====================================================================
// Uniforms

struct Camera {
    projection: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;


//====================================================================

struct VertexIn {
    // Vertex
    @builtin(vertex_index) index: u32,

    // Instance
    @location(0) size: vec2<f32>,
    @location(1) transform_1: vec4<f32>,
    @location(2) transform_2: vec4<f32>,
    @location(3) transform_3: vec4<f32>,
    @location(4) transform_4: vec4<f32>,
    @location(5) color: vec4<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

//====================================================================

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    var vertex_pos: vec2<f32>;

    switch (in.index) {
        // 0 = Top Left
        case 0u: {
            vertex_pos = vec2<f32>(-0.5, 0.5);
            out.uv = vec2(0., 0.);
            break;
        }
        // 1 = Top Right
        case 2u: {
            vertex_pos = vec2<f32>(0.5, 0.5);
            out.uv = vec2<f32>(1., 0.);
            break;
        }
        // Bottom Left
        case 1u: {
            vertex_pos = vec2<f32>(-0.5, -0.5);
            out.uv = vec2<f32>(0., 1.);
            break;
        }
        // Bottom Right
        case 3u: {
            vertex_pos = vec2<f32>(0.5, -0.5);
            out.uv = vec2<f32>(1., 1.);
            break;
        }
        default: {}
    }
    
    let transform = mat4x4<f32>(
        in.transform_1,
        in.transform_2,
        in.transform_3,
        in.transform_4,
    );

    vertex_pos *= in.size;

    out.clip_position =
        camera.projection
        * transform
        * vec4<f32>(vertex_pos, 1., 1.);

    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture, texture_sampler, in.uv);
    
    return tex_color * in.color;
}

//====================================================================


