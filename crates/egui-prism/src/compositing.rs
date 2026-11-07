//! Host-installed isolation of drawing groups before applying opacity.

use std::sync::Arc;

use egui::{Context, Id, PaintCallback, Painter, Rect, Shape, epaint::ClippedShape};

/// A shape or isolated child group, in back-to-front paint order.
#[derive(Clone, Debug)]
pub enum Item {
    /// A shape with its own clip rectangle.
    Shape(ClippedShape),
    /// A child whose opacity is applied before its parent's opacity.
    Group(Group),
}

/// Immutable frame-local drawing data, with no ownership of referenced textures.
#[derive(Clone, Debug)]
pub struct Group {
    bounds: Rect,
    opacity: f32,
    items: Vec<Item>,
}

impl Group {
    /// Assemble a group whose bounds include all desired antialiasing, ring and shadow pixels.
    pub fn new(bounds: Rect, opacity: f32, items: Vec<Item>) -> Result<Self, Error> {
        if !bounds.is_finite() || !bounds.is_positive() {
            return Err(Error::InvalidGroup("bounds must be finite and positive"));
        }
        if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
            return Err(Error::InvalidGroup("opacity must be between zero and one"));
        }
        fn valid_shape(shape: &Shape) -> bool {
            match shape {
                Shape::Callback(_) => false,
                Shape::Vec(shapes) => shapes.iter().all(valid_shape),
                Shape::Mesh(mesh) => {
                    mesh.is_valid()
                        && mesh
                            .vertices
                            .iter()
                            .all(|vertex| vertex.pos.is_finite() && vertex.uv.is_finite())
                }
                _ => true,
            }
        }
        for item in &items {
            if let Item::Shape(shape) = item
                && (!shape.clip_rect.is_finite()
                    || !shape.transform_after_tessellation.scaling.is_finite()
                    || shape.transform_after_tessellation.scaling <= 0.0
                    || !shape.transform_after_tessellation.translation.is_finite()
                    || !valid_shape(&shape.shape))
            {
                return Err(Error::InvalidGroup(
                    "shapes require finite clips, valid meshes and no callbacks",
                ));
            }
        }
        Ok(Self {
            bounds,
            opacity,
            items,
        })
    }

    /// The group's explicit visual extent, in egui points.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    /// The opacity applied once to the assembled contents.
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// The immutable drawing contents in paint order.
    pub fn items(&self) -> &[Item] {
        &self.items
    }
}

/// An explicit submission or host-capability failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// This egui context has no installed compositor.
    MissingHost,
    /// This egui context already has a compositor.
    AlreadyInstalled,
    /// The submitted group is malformed or unsupported.
    InvalidGroup(&'static str),
    /// The graphics context was lost and the renderer must be recreated.
    ContextLost,
    /// The renderer rejected setup or painting.
    Renderer(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHost => f.write_str("group compositing requires an installed host"),
            Self::AlreadyInstalled => f.write_str("group compositing is already installed"),
            Self::InvalidGroup(reason) => write!(f, "invalid drawing group: {reason}"),
            Self::ContextLost => {
                f.write_str("graphics context lost; recreate the renderer and compositor")
            }
            Self::Renderer(reason) => write!(f, "group compositing failed: {reason}"),
        }
    }
}
impl std::error::Error for Error {}

/// A host adapter that turns immutable groups into its renderer's callbacks.
///
/// Implementations must reject unavailable capabilities, revoke old callbacks on
/// shutdown, retain no texture ownership, and report paint-time failures to the host.
pub trait Backend: Send + Sync + 'static {
    /// Prepare a callback without drawing or transferring the host's textures.
    fn callback(&self, context: &Context, group: Group) -> Result<PaintCallback, Error>;
}

#[derive(Clone)]
struct Installed(Arc<dyn Backend>);
fn key() -> Id {
    Id::unique("egui-prism/group-compositor")
}

/// A host's registration, removed when this handle is dropped.
#[must_use = "keep the installation alive until the host shuts down"]
pub struct Installation {
    context: Context,
    backend: Arc<dyn Backend>,
}

/// Install one compositor for this context and return its lifetime handle.
pub fn install(context: &Context, backend: Arc<dyn Backend>) -> Result<Installation, Error> {
    context.data_mut(|data| {
        if data.get_temp::<Installed>(key()).is_some() {
            return Err(Error::AlreadyInstalled);
        }
        data.insert_temp(key(), Installed(backend.clone()));
        Ok(Installation {
            context: context.clone(),
            backend,
        })
    })
}

impl Drop for Installation {
    fn drop(&mut self) {
        self.context.data_mut(|data| {
            if data
                .get_temp::<Installed>(key())
                .is_some_and(|installed| Arc::ptr_eq(&installed.0, &self.backend))
            {
                data.remove::<Installed>(key());
            }
        });
    }
}

/// Submit a group, applying inherited painter opacity once to its final composite.
///
/// Shapes must carry their unfaded colors. The caller retains all image handles
/// through painting. Use nested groups when a parent includes overlapping siblings;
/// multiplying each sibling's opacity is not parent isolation.
pub fn paint(painter: &Painter, mut group: Group) -> Result<(), Error> {
    let backend = painter
        .ctx()
        .data(|data| data.get_temp::<Installed>(key()))
        .ok_or(Error::MissingHost)?;
    group.opacity *= painter.opacity();
    let callback = backend.0.callback(painter.ctx(), group)?;
    painter.add(Shape::Callback(callback));
    Ok(())
}
