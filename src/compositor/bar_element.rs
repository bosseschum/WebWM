use smithay::backend::renderer::{
    element::{Element, Id, Kind, RenderElement, UnderlyingStorage},
    gles::{GlesError, GlesFrame, GlesRenderer, GlesTexture},
    utils::{CommitCounter, DamageSet, OpaqueRegions},
    Frame, ImportMem, Renderer, Texture,
};
use smithay::utils::{Buffer, Physical, Rectangle, Scale, Size, Transform};
use std::sync::Arc;

/// A render element for the status bar
#[derive(Clone)]
pub struct BarRenderElement {
    id: Id,
    geometry: Rectangle<i32, Physical>,
    texture: Arc<GlesTexture>,
    commit_counter: CommitCounter,
}

impl BarRenderElement {
    pub fn new(
        renderer: &mut GlesRenderer,
        buffer: &[u8],
        size: Size<i32, Physical>,
        geometry: Rectangle<i32, Physical>,
    ) -> Result<Self, GlesError> {
        // Import the RGBA buffer as a texture
        let texture = renderer.import_memory(
            buffer,
            smithay::backend::allocator::Fourcc::Argb8888,
            smithay::utils::Size::from((size.w, size.h)),
            false, // Not flipped
        )?;

        Ok(Self {
            id: Id::new(),
            geometry,
            texture: Arc::new(texture),
            commit_counter: CommitCounter::default(),
        })
    }

    pub fn update(
        &mut self,
        renderer: &mut GlesRenderer,
        buffer: &[u8],
        size: Size<i32, Physical>,
    ) -> Result<(), GlesError> {
        // Re-import the buffer as a new texture
        let new_texture = renderer.import_memory(
            buffer,
            smithay::backend::allocator::Fourcc::Argb8888,
            smithay::utils::Size::from((size.w, size.h)),
            false,
        )?;
        self.texture = Arc::new(new_texture);
        self.commit_counter.increment();
        Ok(())
    }
}

impl Element for BarRenderElement {
    fn id(&self) -> &Id {
        &self.id
    }

    fn current_commit(&self) -> CommitCounter {
        self.commit_counter
    }

    fn src(&self) -> Rectangle<f64, Buffer> {
        Rectangle::from_loc_and_size(
            (0.0, 0.0),
            self.texture
                .size()
                .to_logical(1, Transform::Normal)
                .to_f64()
                .to_buffer(1.0, Transform::Normal),
        )
    }

    fn geometry(&self, _scale: Scale<f64>) -> Rectangle<i32, Physical> {
        self.geometry
    }

    fn transform(&self) -> Transform {
        Transform::Normal
    }

    fn damage_since(
        &self,
        _scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> DamageSet<i32, Physical> {
        if commit != Some(self.commit_counter) {
            DamageSet::from_slice(&[self.geometry])
        } else {
            DamageSet::default()
        }
    }

    fn opaque_regions(&self, _scale: Scale<f64>) -> OpaqueRegions<i32, Physical> {
        OpaqueRegions::default()
    }

    fn alpha(&self) -> f32 {
        1.0
    }

    fn kind(&self) -> Kind {
        Kind::Unspecified
    }
}

impl RenderElement<GlesRenderer> for BarRenderElement {
    fn draw(
        &self,
        frame: &mut GlesFrame<'_, '_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
        opaque_regions: &[Rectangle<i32, Physical>],
    ) -> Result<(), GlesError> {
        frame.render_texture_from_to(
            &self.texture,
            src,
            dst,
            damage,
            opaque_regions,
            Transform::Normal,
            1.0, // alpha
            None,
            &[],
        )
    }
}
