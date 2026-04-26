
struct Camera {
    origin: vec4<f32>,
    look_at: vec4<f32>,
    params: vec4<f32>,
};
@group(0) @binding(0) var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@location(0) pos: vec3<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(pos, 1.0);
    out.uv = pos.xy; // 把 NDC 坐标传给片元着色器作为 UV
    return out;
}

// 球体的 SDF 函数：f(p) = |p - c| - r
fn sdf_sphere(p: vec3<f32>, center: vec3<f32>, radius: f32) -> f32 {
    return length(p - center) - radius;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ro = camera.origin.xyz;

    let forward = normalize(camera.look_at.xyz - camera.origin.xyz);
    let world_up = vec3<f32>(0.0, 1.0, 0.0);
    let right = normalize(cross(forward, world_up));
    let up = cross(right, forward);

    let aspect = camera.params.x;
    let fov_y = camera.params.y;
    let scale = tan(fov_y * 0.5);

    let uv = vec2<f32>(
        in.uv.x * aspect * scale,
        in.uv.y * scale
    );

    let rd = normalize(forward + uv.x * right + uv.y * up);

    var t = 0.0;
    let max_dist = 100.0;

    // Raymarching 循环
    for (var i = 0; i < 64; i++) {
        let p = ro + rd * t; // 当前光线到达的位置
        let d1 = sdf_sphere(p, vec3<f32>(0.0, 0.0, 0.0), 1.0); // 定义一个在 (0,0,-5) 半径为 1 的球
        let d2 = sdf_sphere(p, vec3<f32>(1.0, 0.0, -1.0), 1.0);
        let d = min(d1, d2);
        if d < 0.001 {
            // 撞到了！根据深度简单涂个色
            let color = 1.0 - t / 10.0;
            return vec4<f32>(color, color, color, 1.0);
        }

        t += d;
        if t > max_dist { break; }
    }

    // 没撞到，返回背景色（深灰色）
    return vec4<f32>(0.1, 0.1, 0.1, 1.0);
}