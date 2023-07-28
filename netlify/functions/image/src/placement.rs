#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalPlacement{
    Left, Centre, Right
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalPlacement{
    Top, Centre,Bottom
}

impl VerticalPlacement{
    pub fn get_y(&self, full_height: f32, item_height: f32)-> f32{
        match self{
            VerticalPlacement::Top => 0.0,
            VerticalPlacement::Centre => (full_height - item_height) * 0.5,
            VerticalPlacement::Bottom => (full_height - item_height),
        }
    }
}

impl HorizontalPlacement{
    pub fn get_x(&self, full_width: f32, item_width: f32)-> f32{
        match self{
            HorizontalPlacement::Left => 0.0,
            HorizontalPlacement::Centre => (full_width - item_width) * 0.5,
            HorizontalPlacement::Right => (full_width - item_width),
        }
    }
}