// Feature shader

[[block]]
struct CameraUniform {
  view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> camera: CameraUniform;

struct VertexInput {
  [[location(0)]] position: vec3<f32>;
  [[location(1)]] normal: vec3<f32>;
};

struct InstanceInput {
  [[location(2)]] model_0: vec4<f32>;
  [[location(3)]] model_1: vec4<f32>;
  [[location(4)]] model_2: vec4<f32>;
  [[location(5)]] model_3: vec4<f32>;
};

struct VertexOutput {
  [[builtin(position)]] clip_position: vec4<f32>;
  [[location(0), interpolate(perspective)]] normal: vec3<f32>;
};

[[stage(vertex)]]
fn main(
  vertex: VertexInput,
  instance: InstanceInput
) -> VertexOutput {
  var out: VertexOutput;
  let model = mat4x4<f32>(
    instance.model_0,
    instance.model_1,
    instance.model_2,
    instance.model_3,
  );
  out.clip_position = camera.view_proj * model * vec4<f32>(vertex.position, 1.0);
  out.normal = vertex.normal;
  return out;
}

// Fragment shader

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
  let PI = 3.1415926538;
  let LIGHT_DIRECTION = vec3<f32>(0.0, -sin(PI / 6.0), -cos(PI / 6.0));
  let LIGHT_INTENSITY = 1.0;
  let ALBEDO = 1.0;
  let COLOR = vec3<f32>(0.3, 0.2, 0.1);
  let illumination = dot(in.normal, -LIGHT_DIRECTION);
  let illumination = max(0.0, illumination);
  return vec4<f32>(ALBEDO * LIGHT_INTENSITY * illumination * COLOR, 1.0);
}
