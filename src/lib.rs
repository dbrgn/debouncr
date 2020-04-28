//! # Debouncr
//!
//! A simple and efficient `no_std` input debouncer that uses integer bit
//! shifting to debounce inputs. The algorithm can detect rising and falling
//! edges and only requires 1 byte of RAM for detecting up to 8 consecutive
//! high/low states or 2 bytes of RAM for detecting up to 16 consecutive
//! high/low states.
//!
//! The algorithm is based on the [Ganssle Guide to
//! Debouncing](http://www.ganssle.com/debouncing-pt2.htm) (section "An
//! Alternative").
//!
//! ## API
//!
//! ### Instantiate
//!
//! First, decide how many consecutive logical-high states you want to detect.
//! For example, if you poll the input pin every 5 ms and require 4
//! consecutive logical-high states to trigger a debounced press event, that
//! event will happen after 20 ms.
//!
//! ```rust
//! use debouncr::{Debouncer, debounce_4};
//! let mut debouncer = debounce_4(); // Type: Debouncer<u8, Repeat4>
//! ```
//! 
//! ### Update
//!
//! In regular intervals, call the `update(pressed)` function to update the
//! internal state.
//!
//! ```rust
//! # use debouncr::{Debouncer, debounce_4};
//! # let mut debouncer = debounce_4();
//! # fn poll_button() -> bool { true };
//! let is_pressed = poll_button();
//! let rising_edge_detected = debouncer.update(is_pressed);
//! ````
//!
//! The `update` function will only return `true` on a rising edge.
//!
//! ### Query debounced state
//!
//! You can also query the current debounced state. If none of the `n` recent
//! updates were pressed, then the debounced state will be low. If all `n`
//! recent updates were pressed, then the debounced state will be high.
//!
//! ```rust
//! # use debouncr::{Debouncer, debounce_3};
//! let mut debouncer = debounce_3();
//!
//! // Initially low
//! assert!(debouncer.is_low());
//! assert!(!debouncer.is_high());
//!
//! // Update, now it's neither high nor low
//! debouncer.update(true);
//! assert!(!debouncer.is_low());
//! assert!(!debouncer.is_high());
//!
//! // After two more updates, it's high
//! debouncer.update(true);
//! debouncer.update(true);
//! assert!(debouncer.is_high());
//! ```
//!
//! ## Example: RTFM
//!
//! If you want to debounce a pin in an [RTFM](https://rtfm.rs/) project,
//! register a resource and a timer.
//!
//! ```ignore
//! use debouncr::{Debouncer, debounce_12, Repeat12};
//!
//! #[app(..., monotonic = rtfm::cyccnt::CYCCNT)]
//! const APP: () = {
//!     struct Resources {
//!         button: gpioa::PA11<Input<PullUp>>,
//!         button_state: Debounce<u16, Repeat12>,
//!     }
//!
//!     #[init(spawn = [poll_button])]
//!     fn init(ctx: init::Context) -> init::LateResources {
//!         // ...
//!         ctx.spawn.poll_button().unwrap();
//!         init::LateResources {
//!             button,
//!             button_state: debounce_12(),
//!         }
//!     }
//!
//!     /// Regularly called task that polls the buttons and debounces them.
//!     ///
//!     /// The handlers are only called for a rising edge with 12 consecutive high
//!     /// pin inputs. This means that if the interrupt is scheduled every 1 ms
//!     /// and the input pin becomes high, the task will fire after 12 ms. Every
//!     /// low input will reset the whole state though.
//!     #[task(
//!         resources = [button, button_state],
//!         spawn = [button_pressed],
//!         schedule = [poll_button],
//!     )]
//!     fn poll_button(ctx: poll_button::Context) {
//!         // Poll button
//!         let pressed: bool = ctx.resources.button.is_low().unwrap();
//!
//!         // Update state
//!         let rising_edge = ctx.resources.button_state.update(pressed);
//!
//!         // Dispatch event
//!         if rising_edge {
//!             ctx.spawn.button_pressed().unwrap();
//!         }
//!
//!         // Re-schedule the timer interrupt
//!         ctx.schedule
//!             .poll_button(ctx.scheduled + POLL_PERIOD.cycles())
//!             .unwrap();
//!     }
//!
//!     /// The button was pressed.
//!     #[task]
//!     fn button_pressed(ctx: button_pressed::Context) {
//!         // Button was pressed, handle event somehow
//!     }
//!
//! };
//! ```
#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code, missing_docs)]

use doc_comment::doc_comment;

/// The main debouncer instance.
///
/// It wraps a `u8` or `u16`, depending on the number of required consecutive
/// logical-high states.
///
/// To create an instance, use the appropriate `debounce_X` function (where `X`
/// is the number of required consecutive logical-high states).
#[repr(transparent)]
pub struct Debouncer<S, M> {
    state: S,
    mask: core::marker::PhantomData<M>,
}

/// Rising or falling edge.
#[derive(Debug, PartialEq, Eq)]
pub enum Edge {
    /// A rising edge
    Rising,
    /// A falling edge
    Falling,
}

macro_rules! impl_logic {
    ($T:ty, $count:expr, $M:ident, $name:ident, $mask:expr) => {
        doc_comment! {
            concat!(
                "Detect ",
                $count,
                " consecutive logical-high states.\n\n",
                "This type should not be used directly. ",
                "Instead, construct a [`Debouncer`](struct.Debouncer.html) through [`debounce_",
                $count,
                "()`](fn.debounce_",
                $count,
                ".html).",
            ),
            pub struct $M;
        }

        doc_comment! {
            concat!(
                "Create a new debouncer that can detect a rising edge of ",
                $count,
                " consecutive logical-high states.",
            ),
            pub fn $name() -> Debouncer<$T, $M> {
                Debouncer {
                    state: 0,
                    mask: core::marker::PhantomData,
                }
            }
        }

        impl Debouncer<$T, $M> {
            /// Update the state.
            pub fn update(&mut self, pressed: bool) -> Option<Edge> {
                // If all bits are already 1 or 0 and there was no change,
                // we can immediately return.
                if self.state == $mask && pressed {
                    return None;
                }
                if self.state == 0 && !pressed {
                    return None;
                }

                // Update state by shifting in the press state & masking
                self.state = ((self.state << 1) | (pressed as $T)) & $mask;

                // Query updated value
                if self.state == $mask {
                    Some(Edge::Rising)
                } else if self.state == 0 {
                    Some(Edge::Falling)
                } else {
                    None
                }
            }

            /// Return `true` if the debounced state is logical high.
            pub fn is_high(&self) -> bool {
                self.state == $mask
            }

            /// Return `true` if the debounced state is logical low.
            pub fn is_low(&self) -> bool {
                self.state == 0
            }
        }
    };
}

impl_logic!( u8,  2,  Repeat2,  debounce_2,           0b0000_0011);
impl_logic!( u8,  3,  Repeat3,  debounce_3,           0b0000_0111);
impl_logic!( u8,  4,  Repeat4,  debounce_4,           0b0000_1111);
impl_logic!( u8,  5,  Repeat5,  debounce_5,           0b0001_1111);
impl_logic!( u8,  6,  Repeat6,  debounce_6,           0b0011_1111);
impl_logic!( u8,  7,  Repeat7,  debounce_7,           0b0111_1111);
impl_logic!( u8,  8,  Repeat8,  debounce_8,           0b1111_1111);
impl_logic!(u16,  9,  Repeat9,  debounce_9, 0b0000_0001_1111_1111);
impl_logic!(u16, 10, Repeat10, debounce_10, 0b0000_0011_1111_1111);
impl_logic!(u16, 11, Repeat11, debounce_11, 0b0000_0111_1111_1111);
impl_logic!(u16, 12, Repeat12, debounce_12, 0b0000_1111_1111_1111);
impl_logic!(u16, 13, Repeat13, debounce_13, 0b0001_1111_1111_1111);
impl_logic!(u16, 14, Repeat14, debounce_14, 0b0011_1111_1111_1111);
impl_logic!(u16, 15, Repeat15, debounce_15, 0b0111_1111_1111_1111);
impl_logic!(u16, 16, Repeat16, debounce_16, 0b1111_1111_1111_1111);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rising_edge() {
        // Initially not pressed
        let mut debouncer: Debouncer<u8, Repeat3> = debounce_3();
        assert!(debouncer.is_low());

        // Three pressed updates required
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(true), Some(Edge::Rising));

        // Further presses do not indicate a rising edge anymore
        assert_eq!(debouncer.update(true), None);

        // A depressed state resets counting
        assert_eq!(debouncer.update(false), None);
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(true), Some(Edge::Rising));
    }

    #[test]
    fn test_falling_edge() {
        // Initially not pressed
        let mut debouncer: Debouncer<u8, Repeat3> = debounce_3();
        assert!(debouncer.is_low());

        // A single non-pressed update does not trigger
        assert_eq!(debouncer.update(false), None);
        assert!(debouncer.is_low());

        // Trigger a falling edge
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(false), None);
        assert_eq!(debouncer.update(false), None);
        assert_eq!(debouncer.update(false), Some(Edge::Falling));
        assert_eq!(debouncer.update(false), None);
        assert!(debouncer.is_low());
    }

    #[test]
    fn test_debounce_16() {
        // Sixteen pressed updates required
        let mut debouncer: Debouncer<u16, Repeat16> = debounce_16();
        assert!(debouncer.is_low());
        for _ in 0..15 {
            assert_eq!(debouncer.update(true), None);
            assert!(!debouncer.is_high());
        }
        assert_eq!(debouncer.update(true), Some(Edge::Rising));
        assert!(debouncer.is_high());
        assert_eq!(debouncer.update(true), None);
        assert!(debouncer.is_high());
    }

    #[test]
    fn test_is_low_high() {
        // Initially low
        let mut debouncer: Debouncer<u8, Repeat8> = debounce_8();
        assert!(debouncer.is_low());
        assert!(!debouncer.is_high());

        // Depressed updates don't change the situation
        debouncer.update(false);
        assert!(debouncer.is_low());
        assert!(!debouncer.is_high());

        // A pressed update causes neither low nor high state
        for _ in 0..7 {
            assert!(debouncer.update(true).is_none());
            assert!(!debouncer.is_low());
            assert!(!debouncer.is_high());
        }

        // Once complete, the state is high
        assert_eq!(debouncer.update(true), Some(Edge::Rising));
        assert!(!debouncer.is_low());
        assert!(debouncer.is_high());

        // Consecutive pressed updates don't trigger an edge but are still high
        assert!(debouncer.update(true).is_none());
        assert!(!debouncer.is_low());
        assert!(debouncer.is_high());
    }

    /// Ensure the promised low RAM consumption.
    #[test]
    fn test_ram_consumption() {
        assert_eq!(std::mem::size_of_val(&debounce_2()), 1);
        assert_eq!(std::mem::size_of_val(&debounce_8()), 1);
        assert_eq!(std::mem::size_of_val(&debounce_9()), 2);
        assert_eq!(std::mem::size_of_val(&debounce_16()), 2);
    }
}
