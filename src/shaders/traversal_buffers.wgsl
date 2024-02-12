@group(0) @binding(0) var<storage, read> scene: Scene;
@group(0) @binding(1) var<storage, read> rays: RayBuffer;
@group(0) @binding(2) var<storage, read> bvhBuffer: BVHBuffer;
@group(0) @binding(3) var<storage, read> triangleBuffer: TriangleBuffer;
@group(0) @binding(4) var<storage, read> quadLightBuffer: QuadLightBuffer;
@group(0) @binding(5) var t_sky: texture_2d<f32>;
@group(0) @binding(6) var s_sky: sampler;