// Feature shader

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
  [[location(6)]] color: vec3<f32>;
};

struct VertexOutput {
  [[builtin(position)]] clip_position: vec4<f32>;
  [[location(0), interpolate(perspective)]] normal: vec3<f32>;
  [[location(1)]] color: vec3<f32>;
};

[[stage(vertex)]]
fn vertex(
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
  out.color = instance.color;
  return out;
}

// Fragment shader

[[stage(fragment)]]
fn fragment(vertex: VertexOutput) -> [[location(0)]] vec4<f32> {
  let PI = 3.1415926538;
  let Y = -sin(PI / 6.0);
  let Z = cos(PI / 6.0);
  let LIGHT_DIRECTION = vec3<f32>(0.0, Y, Z);
  let LIGHT_INTENSITY = 1.0;
  let ALBEDO = 1.0;
  let illumination = dot(vertex.normal, -LIGHT_DIRECTION);
  let illumination = max(0.0, illumination);
  return vec4<f32>(ALBEDO * LIGHT_INTENSITY * illumination * vertex.color, 1.0);
}
