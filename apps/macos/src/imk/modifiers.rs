//! 当前修饰键状态与单击 Shift 跟踪。

use objc2_app_kit::{NSEvent, NSEventModifierFlags};

/// Caps Lock 亮着。它只管字母大小写，不再代表中 / 英文模式。
pub fn caps_lock_on() -> bool {
    NSEvent::modifierFlags_class().contains(NSEventModifierFlags::CapsLock)
}

pub fn shift_down() -> bool {
    NSEvent::modifierFlags_class().contains(NSEventModifierFlags::Shift)
}

/// 左右 Shift 的硬件键码。
const SHIFT_KEYS: [u16; 2] = [56, 60];

/// 跟踪一次 Shift 按下是“单击”还是与其他键组合。
#[derive(Default)]
pub struct ShiftTap {
    /// 正在候选单击的 Shift 键码。
    pressed: Option<u16>,

    /// Shift 按住期间是否用过其他键。
    used: bool,
}

impl ShiftTap {
    /// 修饰键变化。只有 Shift 独立按下又松开才返回 `true`。
    pub fn flags_changed(
        &mut self,
        key_code: u16,
        shift_down: bool,
        other_modifier_down: bool,
    ) -> bool {
        if !SHIFT_KEYS.contains(&key_code) {
            if self.pressed.is_some() {
                self.used = true;
            }
            return false;
        }
        if shift_down {
            match self.pressed {
                None if !other_modifier_down => {
                    self.pressed = Some(key_code);
                    self.used = false;
                }
                Some(pressed) if pressed != key_code => self.used = true,
                _ => {}
            }
            return false;
        }
        let tapped = self.pressed == Some(key_code) && !self.used && !other_modifier_down;
        self.pressed = None;
        self.used = false;
        tapped
    }

    /// Shift 按住时收到普通按键，这一轮不再算单击。
    pub fn note_key_down(&mut self) {
        if self.pressed.is_some() {
            self.used = true;
        }
    }
}

/// 按 Caps Lock 灯和 Shift 状态得到英文模式要上屏的字母。
pub fn english_letter(letter: char, caps_lock: bool, shift: bool) -> char {
    if caps_lock || shift {
        letter.to_ascii_uppercase()
    } else {
        letter.to_ascii_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_tap_toggles_only_after_press_and_release() {
        let mut tap = ShiftTap::default();
        assert!(!tap.flags_changed(56, true, false));
        assert!(tap.flags_changed(56, false, false));
    }

    #[test]
    fn shift_with_a_letter_does_not_toggle() {
        let mut tap = ShiftTap::default();
        tap.flags_changed(56, true, false);
        tap.note_key_down();
        assert!(!tap.flags_changed(56, false, false));
    }

    #[test]
    fn shift_with_another_modifier_does_not_toggle() {
        let mut tap = ShiftTap::default();
        tap.flags_changed(56, true, false);
        tap.flags_changed(58, true, true);
        assert!(!tap.flags_changed(56, false, true));
    }

    #[test]
    fn caps_lock_controls_english_letter_case() {
        assert_eq!(english_letter('A', false, false), 'a');
        assert_eq!(english_letter('a', false, true), 'A');
        assert_eq!(english_letter('a', true, false), 'A');
        assert_eq!(english_letter('a', true, true), 'A');
    }
}
