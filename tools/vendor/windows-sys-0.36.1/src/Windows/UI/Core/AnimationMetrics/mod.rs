pub type AnimationDescription = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Core_AnimationMetrics\"`*"]
#[repr(transparent)]
pub struct AnimationEffect(pub i32);
impl AnimationEffect {
    pub const Expand: Self = Self(0i32);
    pub const Collapse: Self = Self(1i32);
    pub const Reposition: Self = Self(2i32);
    pub const FadeIn: Self = Self(3i32);
    pub const FadeOut: Self = Self(4i32);
    pub const AddToList: Self = Self(5i32);
    pub const DeleteFromList: Self = Self(6i32);
    pub const AddToGrid: Self = Self(7i32);
    pub const DeleteFromGrid: Self = Self(8i32);
    pub const AddToSearchGrid: Self = Self(9i32);
    pub const DeleteFromSearchGrid: Self = Self(10i32);
    pub const AddToSearchList: Self = Self(11i32);
    pub const DeleteFromSearchList: Self = Self(12i32);
    pub const ShowEdgeUI: Self = Self(13i32);
    pub const ShowPanel: Self = Self(14i32);
    pub const HideEdgeUI: Self = Self(15i32);
    pub const HidePanel: Self = Self(16i32);
    pub const ShowPopup: Self = Self(17i32);
    pub const HidePopup: Self = Self(18i32);
    pub const PointerDown: Self = Self(19i32);
    pub const PointerUp: Self = Self(20i32);
    pub const DragSourceStart: Self = Self(21i32);
    pub const DragSourceEnd: Self = Self(22i32);
    pub const TransitionContent: Self = Self(23i32);
    pub const Reveal: Self = Self(24i32);
    pub const Hide: Self = Self(25i32);
    pub const DragBetweenEnter: Self = Self(26i32);
    pub const DragBetweenLeave: Self = Self(27i32);
    pub const SwipeSelect: Self = Self(28i32);
    pub const SwipeDeselect: Self = Self(29i32);
    pub const SwipeReveal: Self = Self(30i32);
    pub const EnterPage: Self = Self(31i32);
    pub const TransitionPage: Self = Self(32i32);
    pub const CrossFade: Self = Self(33i32);
    pub const Peek: Self = Self(34i32);
    pub const UpdateBadge: Self = Self(35i32);
}
impl ::core::marker::Copy for AnimationEffect {}
impl ::core::clone::Clone for AnimationEffect {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"UI_Core_AnimationMetrics\"`*"]
#[repr(transparent)]
pub struct AnimationEffectTarget(pub i32);
impl AnimationEffectTarget {
    pub const Primary: Self = Self(0i32);
    pub const Added: Self = Self(1i32);
    pub const Affected: Self = Self(2i32);
    pub const Background: Self = Self(3i32);
    pub const Content: Self = Self(4i32);
    pub const Deleted: Self = Self(5i32);
    pub const Deselected: Self = Self(6i32);
    pub const DragSource: Self = Self(7i32);
    pub const Hidden: Self = Self(8i32);
    pub const Incoming: Self = Self(9i32);
    pub const Outgoing: Self = Self(10i32);
    pub const Outline: Self = Self(11i32);
    pub const Remaining: Self = Self(12i32);
    pub const Revealed: Self = Self(13i32);
    pub const RowIn: Self = Self(14i32);
    pub const RowOut: Self = Self(15i32);
    pub const Selected: Self = Self(16i32);
    pub const Selection: Self = Self(17i32);
    pub const Shown: Self = Self(18i32);
    pub const Tapped: Self = Self(19i32);
}
impl ::core::marker::Copy for AnimationEffectTarget {}
impl ::core::clone::Clone for AnimationEffectTarget {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IPropertyAnimation = *mut ::core::ffi::c_void;
pub type OpacityAnimation = *mut ::core::ffi::c_void;
pub type PropertyAnimation = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"UI_Core_AnimationMetrics\"`*"]
#[repr(transparent)]
pub struct PropertyAnimationType(pub i32);
impl PropertyAnimationType {
    pub const Scale: Self = Self(0i32);
    pub const Translation: Self = Self(1i32);
    pub const Opacity: Self = Self(2i32);
}
impl ::core::marker::Copy for PropertyAnimationType {}
impl ::core::clone::Clone for PropertyAnimationType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ScaleAnimation = *mut ::core::ffi::c_void;
pub type TranslationAnimation = *mut ::core::ffi::c_void;
