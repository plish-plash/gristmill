use super::{RenderSubpass, SubpassSetup, SubpassContents, AutoCommandBufferBuilder, QueueFamily, DynamicState, Size};

// ------------------------------------------------------------------------------------------------

pub struct Optional<T> where T: RenderSubpass {
    inner: Option<T>
}

impl<T> Optional<T> where T: RenderSubpass {
    pub fn create_inner(&mut self, subpass_setup: &mut SubpassSetup) {
        if self.inner.is_some() {
            panic!("inner subpass already exists");
        }
        self.inner = Some(T::new(subpass_setup));
    }
    pub fn destroy_inner(&mut self) {
        if self.inner.is_none() {
            panic!("inner subpass does not exist");
        }
        self.inner = None;
    }
}

impl<T> RenderSubpass for Optional<T> where T: RenderSubpass {
    type SubpassCategory = T::SubpassCategory;
    type Scene = Option<T::Scene>;
    fn contents() -> SubpassContents { SubpassContents::Inline }
    fn new(_subpass_setup: &mut SubpassSetup) -> Self {
        Optional { inner: None }
    }
    fn set_dimensions(&mut self, dimensions: Size) {
        if let Some(inner) = self.inner.as_mut() {
            inner.set_dimensions(dimensions);
        }
    }
    fn pre_render(&mut self, scene: &mut Self::Scene, builder: &mut AutoCommandBufferBuilder, queue_family: QueueFamily) {
        if let Some(inner) = self.inner.as_mut() {
            inner.pre_render(scene.as_mut().expect("Scene must be Some if optional subpass is Some"), builder, queue_family);
        }
    }
    fn render(&mut self, scene: &Self::Scene, builder: &mut AutoCommandBufferBuilder, dynamic_state: &DynamicState) {
        if let Some(inner) = self.inner.as_mut() {
            inner.render(scene.as_ref().expect("Scene must be Some if optional subpass is Some"), builder, dynamic_state);
        }
    }
}
