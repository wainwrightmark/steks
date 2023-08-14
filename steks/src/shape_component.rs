use strum::EnumIs;

pub use crate::prelude::*;

#[derive(Component, Debug, Clone, PartialEq, EnumIs)]
pub enum ShapeComponent {
    Free,
    Locked,
    Fixed,
    Void,
    Dragged(Dragged),
}

impl From<&ShapeComponent> for ShapeState {
    fn from(val: &ShapeComponent) -> Self {
        match val {
            ShapeComponent::Free => ShapeState::Normal,
            ShapeComponent::Locked => ShapeState::Locked,
            ShapeComponent::Fixed => ShapeState::Fixed,
            ShapeComponent::Void => ShapeState::Void,
            ShapeComponent::Dragged(_) => ShapeState::Normal,
        }
    }
}

impl From<ShapeState> for ShapeComponent {
    fn from(val: ShapeState) -> Self {
        match val {
            ShapeState::Normal => ShapeComponent::Free,
            ShapeState::Locked => ShapeComponent::Locked,
            ShapeState::Fixed => ShapeComponent::Fixed,
            ShapeState::Void => ShapeComponent::Void,
        }
    }
}

impl ShapeComponent {
    pub fn touch_id(&self) -> Option<u64> {
        let ShapeComponent::Dragged(dragged) = self else {return  None;};
        dragged.drag_source.touch_id()
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
