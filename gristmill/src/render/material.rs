use crate::{render::texture::Texture, render::RenderContext};
use std::{collections::HashMap, sync::Arc};
use vulkano::{
    buffer::{
        cpu_pool::CpuBufferPoolChunk, BufferAccessObject, BufferContents, BufferUsage,
        CpuBufferPool, TypedBufferAccess,
    },
    descriptor_set::DescriptorSetWithOffsets,
    memory::allocator::{MemoryUsage, StandardMemoryAllocator},
    pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint},
    sampler::Sampler,
};

pub trait MaterialPipeline {
    fn pipeline(&self) -> &Arc<GraphicsPipeline>;
    fn texture_descriptor_set(
        &self,
        context: &mut RenderContext,
        texture: Texture,
        sampler: Arc<Sampler>,
    ) -> DescriptorSetWithOffsets;
}

struct InstanceBuffer<I>
where
    [I]: BufferContents,
{
    instances: Vec<I>,
    retain: bool,
    changed: bool,
    buffer: Option<Arc<CpuBufferPoolChunk<I>>>,
    descriptor_set: DescriptorSetWithOffsets,
}

impl<I> InstanceBuffer<I>
where
    [I]: BufferContents,
{
    fn new<P>(context: &mut RenderContext, pipeline: &P, texture: Texture) -> Self
    where
        P: MaterialPipeline,
    {
        let sampler = Sampler::new(context.device(), Default::default()).unwrap();
        InstanceBuffer {
            instances: Vec::new(),
            retain: false,
            changed: false,
            buffer: None,
            descriptor_set: pipeline.texture_descriptor_set(context, texture, sampler),
        }
    }
    fn load(&mut self, buffer_pool: &CpuBufferPool<I>) {
        if self.retain && !self.changed {
            return;
        }
        self.changed = false;
        if !self.instances.is_empty() {
            self.buffer = Some(buffer_pool.from_iter(self.instances.drain(..)).unwrap());
        } else {
            self.buffer = None;
        }
    }
}

pub struct MaterialCache<P, I>
where
    P: MaterialPipeline,
    I: Send + Sync,
    [I]: BufferContents,
{
    pipeline: P,
    buffer_pool: CpuBufferPool<I>,
    cache: HashMap<Texture, InstanceBuffer<I>>,
}

impl<P, I> MaterialCache<P, I>
where
    P: MaterialPipeline,
    I: Send + Sync,
    [I]: BufferContents,
{
    pub fn new(pipeline: P, allocator: Arc<StandardMemoryAllocator>) -> Self {
        MaterialCache {
            pipeline,
            buffer_pool: CpuBufferPool::new(
                allocator,
                BufferUsage {
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                MemoryUsage::Upload,
            ),
            cache: HashMap::new(),
        }
    }

    pub fn pipeline(&self) -> &P {
        &self.pipeline
    }
    pub fn pipeline_mut(&mut self) -> &mut P {
        &mut self.pipeline
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
    pub fn remove(&mut self, texture: &Texture) {
        self.cache.remove(texture);
    }

    fn get_mut(&mut self, context: &mut RenderContext, texture: Texture) -> &mut InstanceBuffer<I> {
        self.cache
            .entry(texture.clone())
            .or_insert_with(|| InstanceBuffer::new(context, &self.pipeline, texture))
    }
    pub fn set_retain_instances(
        &mut self,
        context: &mut RenderContext,
        texture: Texture,
        retain: bool,
    ) {
        self.get_mut(context, texture).retain = retain;
    }
    pub fn queue(&mut self, context: &mut RenderContext, texture: Texture, instance: I) {
        let instance_buffer = self.get_mut(context, texture);
        instance_buffer.changed = true;
        instance_buffer.instances.push(instance);
    }
    pub fn queue_all<It>(&mut self, context: &mut RenderContext, texture: Texture, iter: It)
    where
        It: IntoIterator<Item = I>,
    {
        let instance_buffer = self.get_mut(context, texture);
        instance_buffer.changed = true;
        instance_buffer.instances.extend(iter);
    }

    pub fn draw_all<V>(&mut self, context: &mut RenderContext, vertex_buffer: V, vertex_count: u32)
    where
        V: BufferAccessObject + Clone,
    {
        context
            .builder()
            .bind_pipeline_graphics(self.pipeline.pipeline().clone());

        for instances in self.cache.values_mut() {
            instances.load(&self.buffer_pool);
            if let Some(instance_buffer) = instances.buffer.as_ref() {
                let instance_count = instance_buffer.len();
                context
                    .builder()
                    .bind_vertex_buffers(0, (vertex_buffer.clone(), instance_buffer.clone()))
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.pipeline.pipeline().layout().clone(),
                        0,
                        instances.descriptor_set.clone(),
                    )
                    .draw(vertex_count, instance_count as u32, 0, 0)
                    .unwrap();
            }
        }
    }
}

impl<P, I> MaterialCache<P, I>
where
    P: MaterialPipeline + Clone,
    I: Send + Sync,
    [I]: BufferContents,
{
    pub fn new_shared(&self) -> Self {
        MaterialCache {
            pipeline: self.pipeline.clone(),
            buffer_pool: self.buffer_pool.clone(),
            cache: HashMap::new(),
        }
    }
}
