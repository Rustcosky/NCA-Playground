@group(0) @binding(0)
var texture_in: texture_storage_2d<rgba8unorm, read>;

@group(0) @binding(1)
var texture_out: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(2)
var<uniform> filter_red: mat3x3f;
@group(0) @binding(3)
var<uniform> filter_green: mat3x3f;
@group(0) @binding(4)
var<uniform> filter_blue: mat3x3f;

fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    return state;
}

fn randomFloat(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}

@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let loc = vec2<i32>(invocation_id.xy);
    let dims = textureDimensions(texture_in);
    let total_pixels = dims.x * dims.y;

    let random_red = randomFloat(invocation_id.y * dims.x + invocation_id.x);
    let random_green = randomFloat(total_pixels + invocation_id.y * dims.x + invocation_id.x);
    let random_blue = randomFloat(u32(2) * total_pixels + invocation_id.y * dims.x + invocation_id.x);
    let color = vec4<f32>(random_red, random_green, random_blue, 1.0);

    textureStore(texture_out, loc, color);
}

fn get_cell(loc: vec2<i32>, offset_x: i32, offset_y: i32) -> vec3<f32> {
    let dims = vec2<i32>(textureDimensions(texture_in));
    var offset_loc = (loc + vec2<i32>(offset_x, offset_y) + dims) % dims;
    let value: vec4<f32> = textureLoad(texture_in, offset_loc);
    return value.xyz;
}

fn nca_step(loc: vec2<i32>) -> vec3<f32> {
    var new_val = vec3<f32>(0., 0., 0.);
    for (var i: i32 = -1; i <= 1; i++) {
        for (var j: i32 = -1; j <= 1; j++) {
            new_val[0] += get_cell(loc, i, j)[0] * filter_red[i+1][j+1];
            new_val[1] += get_cell(loc, i, j)[1] * filter_green[i+1][j+1];
            new_val[2] += get_cell(loc, i, j)[2] * filter_blue[i+1][j+1];
        }
    }
    return new_val;
}

fn activation_fn_red(x: f32) -> f32 {
	return -1./pow(2., (0.6*pow(x, 2.)))+1.;
}

fn activation_fn_green(x: f32) -> f32 {
	return abs(x);
}

fn activation_fn_blue(x: f32) -> f32 {
	return abs(1.2*x);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let loc = vec2<i32>(invocation_id.xy);
    let val = nca_step(loc);
    let color = vec4<f32>(
        clamp(activation_fn_red(val[0]), 0., 1.),
        clamp(activation_fn_green(val[1]), 0., 1.),
        clamp(activation_fn_blue(val[2]), 0., 1.),
        1.,
    );
    textureStore(texture_out, loc, color);
}
