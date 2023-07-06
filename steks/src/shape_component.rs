pub use crate::prelude::*;

#[derive(Component, Debug, Clone, PartialEq)]
pub enum ShapeComponent {
    Free,
    Locked,
    Fixed,
    Void,
    Dragged(Dragged),
}

impl Into<ShapeState> for &ShapeComponent {
    fn into(self) -> ShapeState {
        match self {
            ShapeComponent::Free => ShapeState::Normal,
            ShapeComponent::Locked => ShapeState::Locked,
            ShapeComponent::Fixed => ShapeState::Fixed,
            ShapeComponent::Void => ShapeState::Void,
            ShapeComponent::Dragged(_) => ShapeState::Normal,
        }
    }
}

impl Into<ShapeComponent> for ShapeState {
    fn into(self) -> ShapeComponent {
        match self {
            ShapeState::Normal => ShapeComponent::Free,
            ShapeState::Locked => ShapeComponent::Locked,
            ShapeState::Fixed => ShapeComponent::Fixed,
            ShapeState::Void => ShapeComponent::Void,
        }
    }
}

impl ShapeComponent {
    pub fn is_dragged(&self) -> bool {
        matches!(self, ShapeComponent::Dragged { .. })
    }

    pub fn touch_id(&self) -> Option<u64> {
        let ShapeComponent::Dragged(dragged) = self else {return  None;};
        dragged.drag_source.touch_id()
    }

    pub fn is_free(&self) -> bool {
        matches!(self, ShapeComponent::Free)
    }

    pub fn is_locked(&self) -> bool {
        matches!(self, ShapeComponent::Locked)
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, ShapeComponent::Fixed)
    }

    pub fn is_void(&self) -> bool {
        matches!(self, ShapeComponent::Void)
    }

    pub fn has_drag_source(&self, drag_source: DragSource) -> bool {
        let ShapeComponent::Dragged(dragged) = self else {return  false;};
        dragged.drag_source == drag_source
    }

    pub fn get_offset(&self) -> Vec2 {
        let ShapeComponent::Dragged(dragged) = self else {return  Default::default();};
        dragged.offset
    }
}
