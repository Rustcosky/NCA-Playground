struct PushConstants {
    draw_start: vec2<f32>,
    draw_end: vec2<f32>,
    brush_size: f32,
    brush_type: u32,
    brush_color: array<f32, 3>,
}
var<push_constant> pc: PushConstants;

@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

@compute @workgroup_size(8, 8, 1)
fn draw(@builtin(global_invocation_id) invocation_id: vec3<u32>)
{
    let pixel = vec2<u32>(invocation_id.xy);
    let dims = vec2<u32>(textureDimensions(texture));
    if (pixel.x >= dims.x && pixel.y >= dims.y) {
        return ;
    }

    if (pc.brush_size > 0.0) {
        let pos = vec2<f32>(pixel);
        let point_on_line = closest_point_on_line(pc.draw_start, pc.draw_end, pos);
        switch pc.brush_type {
            case 0u: {
                draw_particle_circle(
                    pos,
                    point_on_line,
                    pc.brush_size,
                    vec4<f32>(pc.brush_color[0], pc.brush_color[1], pc.brush_color[2], 1.)
                );
            }
            case 1u: {
                draw_particle_square(
                    pos,
                    point_on_line,
                    pc.brush_size,
                    vec4<f32>(pc.brush_color[0], pc.brush_color[1], pc.brush_color[2], 1.)
                );
            }
            default: {}
        }
        
    }
}

fn draw_particle_circle(pos: vec2<f32>, draw_pos: vec2<f32>, radius: f32, color: vec4<f32>) {
    let y_start = draw_pos.y - radius;
    let y_end = draw_pos.y + radius;
    let x_start = draw_pos.x - radius;
    let x_end = draw_pos.x + radius;
    if (pos.x >= x_start && pos.x <= x_end && pos.y >= y_start && pos.y <= y_end) {
        let diff = pos - draw_pos;
        let dist = length(diff);
        if (round(dist) <= radius) {
            textureStore(texture, vec2<i32>(pos), color);
        }
    }
}

fn draw_particle_square(pos: vec2<f32>, draw_pos: vec2<f32>, radius: f32, color: vec4<f32>) {
    let y_start = draw_pos.y - radius;
    let y_end = draw_pos.y + radius;
    let x_start = draw_pos.x - radius;
    let x_end = draw_pos.x + radius;
    if (pos.x >= x_start && pos.x <= x_end && pos.y >= y_start && pos.y <= y_end) {
        let diff = pos - draw_pos;
        textureStore(texture, vec2<i32>(pos), color);
    }
}

fn closest_point_on_line(v: vec2<f32>, w: vec2<f32>, p: vec2<f32>) -> vec2<f32> {
    let c = v - w;

    let l2 = dot(c, c);
    if (l2 == 0.0) {
        return v;
    }

    let t = max(0.0, min(1.0, dot(p - v, w - v) / l2));
    let projection = v + t * (w - v);
    return projection;
}