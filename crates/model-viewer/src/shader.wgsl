struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 1.0);
    output.normal = input.normal;
    output.color = input.color;
    return output;
}

@fragment
fn fragment_main(
    input: VertexOutput,
    @builtin(front_facing) front_facing: bool,
) -> @location(0) vec4<f32> {
    let normal_length = length(input.normal);
    if normal_length < 0.0001 {
        return input.color;
    }

    var normal = normalize(input.normal);
    if !front_facing {
        normal = -normal;
    }
    let key = max(dot(normal, normalize(vec3<f32>(-0.35, 0.55, 0.76))), 0.0);
    let fill = max(dot(normal, normalize(vec3<f32>(0.65, -0.35, 0.20))), 0.0);
    let rim = pow(1.0 - abs(normal.z), 2.0);
    let light = clamp(0.30 + 0.58 * key + 0.18 * fill + 0.20 * rim, 0.28, 1.18);
    return vec4<f32>(input.color.rgb * light, input.color.a);
}
