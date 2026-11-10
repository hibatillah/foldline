/// The one thing under the pointer on a page, such as a button or a row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoverTarget<T: PartialEq + Copy> {
    current: Option<T>,
}

impl<T: PartialEq + Copy> Default for HoverTarget<T> {
    fn default() -> Self {
        Self { current: None }
    }
}

impl<T: PartialEq + Copy> HoverTarget<T> {
    pub fn get(&self) -> Option<T> {
        self.current
    }

    pub fn is(&self, target: T) -> bool {
        self.current == Some(target)
    }

    /// Replaces the target; true when it changed, so the caller repaints once.
    pub fn set(&mut self, target: Option<T>) -> bool {
        let changed = self.current != target;
        self.current = target;
        changed
    }

    /// Removes the target; true when there was one.
    pub fn clear(&mut self) -> bool {
        self.set(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_the_same_target_reports_no_change() {
        let mut hover = HoverTarget::default();
        assert!(hover.set(Some(3)));
        assert!(!hover.set(Some(3)));
        assert!(hover.set(Some(4)));
        assert!(hover.is(4));
        assert!(!hover.is(3));
        assert_eq!(hover.get(), Some(4));
    }

    #[test]
    fn clearing_an_empty_target_reports_no_change() {
        let mut hover: HoverTarget<u8> = HoverTarget::default();
        assert!(!hover.clear());
        assert!(!hover.set(None));
        hover.set(Some(1));
        assert!(hover.clear());
        assert_eq!(hover.get(), None);
        assert!(!hover.clear());
    }
}
