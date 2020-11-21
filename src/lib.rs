//! # Debouncr
//!
//! A simple and efficient `no_std` input debouncer that uses integer bit
//! shifting to debounce inputs. The basic algorithm can detect rising and
//! falling edges and only requires 1 byte of RAM for detecting up to
//! 8 consecutive high/low states or 2 bytes of RAM for detecting up to
//! 16 consecutive high/low states.
//!
//! While the regular algorithm will detect any change from "bouncing"
//! to "stable high/low" as an edge, there is also a variant that will
//! only detect changes from "stable high" to "stable low" and
//! vice versa as an edge (see section "Stateful Debouncing").
//!
//! The algorithm is based on the [Ganssle Guide to
//! Debouncing](http://www.ganssle.com/debouncing-pt2.htm) (section "An
//! Alternative").
//!
//! ## API
//!
//! ### Instantiate
//!
//! First, decide how many consecutive states you want to detect.  For example,
//! if you poll the input pin every 5 ms and require 4 consecutive logical-high
//! states to trigger a debounced press event, that event will happen after 20 ms.
//!
//! On initialization, you also need to specify the initial state: `true` for
//! logical-high, `false` for logical-low.
//!
//! ```rust
//! use debouncr::debounce_4;
//!
//! let mut debouncer = debounce_4(false); // Type: Debouncer<u8, Repeat4>
//! ```
//!
//! ### Update
//!
//! In regular intervals, call the `update(pressed)` function to update the
//! internal state.
//!
//! ```rust
//! use debouncr::{debounce_3, Edge};
//!
//! let mut debouncer = debounce_3(false);
//! # fn poll_button() -> bool { true };
//! assert_eq!(debouncer.update(poll_button()), None);
//! assert_eq!(debouncer.update(poll_button()), None);
//! assert_eq!(debouncer.update(poll_button()), Some(Edge::Rising));
//! ````
//!
//! The `update` function will return a rising/falling edge, or `None` if the
//! input is still bouncing.
//!
//! ### Query Debounced State
//!
//! You can also query the current debounced state. If none of the `n` recent
//! updates were pressed, then the debounced state will be low. If all `n`
//! recent updates were pressed, then the debounced state will be high.
//!
//! ```rust
//! use debouncr::debounce_3;
//!
//! let mut debouncer = debounce_3(false);
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
//! ### Stateful Debouncing
//!
//! By default, the debouncer will report any change from "bouncing" to
//! "stable high/low" as an edge. If instead you want to detect only
//! changes from a stable state to the opposite stable state, use the
//! stateful debouncer instead. It has slightly higher (but still tiny) memory
//! overhead than the regular debouncer, because it also stores the previous
//! state in addition to the debouncing updates.
//!
//! ```rust
//! use debouncr::{debounce_stateful_3, Edge};
//!
//! let mut debouncer = debounce_stateful_3(false);
//!
//! // Ensure initial low state
//! assert!(debouncer.is_low());
//!
//! // Temporary bouncing states will not trigger an edge
//! assert_eq!(debouncer.update(true), None);
//! assert_eq!(debouncer.update(false), None);
//! assert_eq!(debouncer.update(false), None);
//! assert_eq!(debouncer.update(false), None);
//!
//! // However, stable opposite states will trigger an edge
//! assert_eq!(debouncer.update(true), None);
//! assert_eq!(debouncer.update(true), None);
//! assert_eq!(debouncer.update(true), Some(Edge::Rising));
//! ```
//!
//! ## Example: RTIC
//!
//! If you want to debounce a pin in an [RTIC](https://rtic.rs/) project,
//! register a resource and a timer.
//!
//! ```ignore
//! use debouncr::{Debouncer, debounce_12, Edge, Repeat12};
//!
//! #[app(..., monotonic = rtic::cyccnt::CYCCNT)]
//! const APP: () = {
//!     struct Resources {
//!         button: gpioa::PA11<Input<PullUp>>,
//!         button_state: Debouncer<u16, Repeat12>,
//!     }
//!
//!     #[init(spawn = [poll_button])]
//!     fn init(ctx: init::Context) -> init::LateResources {
//!         // ...
//!         ctx.spawn.poll_button().unwrap();
//!         init::LateResources {
//!             button,
//!             button_state: debounce_12(false),
//!         }
//!     }
//!
//!     /// Regularly called task that polls the buttons and debounces them.
//!     #[task(
//!         resources = [button, button_state],
//!         spawn = [button_pressed, button_released],
//!         schedule = [poll_button],
//!     )]
//!     fn poll_button(ctx: poll_button::Context) {
//!         // Poll button
//!         let pressed: bool = ctx.resources.button.is_low().unwrap();
//!
//!         // Update state
//!         let edge = ctx.resources.button_state.update(pressed);
//!
//!         // Dispatch event
//!         if edge == Some(Edge::Rising) {
//!             ctx.spawn.button_pressed().unwrap();
//!         } else if edge == Some(Edge::Falling) {
//!             ctx.spawn.button_released().unwrap();
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
//!     /// The button was released.
//!     #[task]
//!     fn button_released(ctx: button_pressed::Context) {
//!         // Button was released, handle event somehow
//!     }
//!
//! };
//! ```
//!
//! ## Memory Consumption
//!
//! Memory size of a debouncer instance:
//!
//! |Debouncer|Repetitions|Bytes|
//! |--|--|--|
//! |[`Debouncer`]|2..8|1|
//! |[`DebouncerStateful`]|2..8|2|
//! |[`Debouncer`]|9..16|2|
//! |[`DebouncerStateful`]|9..16|4|
//!
//! [`Debouncer`]: struct.Debouncer.html
//! [`DebouncerStateful`]: struct.DebouncerStateful.html
#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code, missing_docs)]

use doc_comment::doc_comment;

/// A debouncer.
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

/// A stateful debouncer.
///
/// The regular [`Debouncer`](struct.Debouncer.html) will report any change
/// from "bouncing" to "stable high/low" as an edge. That means that if a
/// button is not pressed, bounces twice and then goes back to unpressed, it
/// will report a falling edge even though there was no rising edge.
///
/// This `DebouncerStateful` on the other hand stores the previous stable state
/// and will only report a falling edge if there was previously a rising edge
/// (and vice versa).
///
/// The memory cost for this is storing an extra enum value per debouncer.
pub struct DebouncerStateful<S, M> {
    debouncer: Debouncer<S, M>,
    last_edge: Edge,
}

/// Rising or falling edge.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Edge {
    /// A rising edge
    Rising,
    /// A falling edge
    Falling,
}

macro_rules! impl_logic {
    ($T:ty, $count:expr, $M:ident, $name:ident, $name_stateful:ident, $mask:expr) => {
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
                "Create a new debouncer that can detect a rising or falling edge of ",
                $count,
                " consecutive logical states.",
            ),
            pub fn $name(initial_state_pressed: bool) -> Debouncer<$T, $M> {
                Debouncer {
                    state: if initial_state_pressed { $mask } else { 0 },
                    mask: core::marker::PhantomData,
                }
            }
        }

        doc_comment! {
            concat!(
                "Create a new stateful debouncer that can detect stable state changes after ",
                $count,
                " consecutive logical states.",
            ),
            pub fn $name_stateful(initial_state_pressed: bool) -> DebouncerStateful<$T, $M> {
                DebouncerStateful {
                    debouncer: $name(initial_state_pressed),
                    last_edge: if initial_state_pressed {Edge::Rising} else {Edge::Falling},
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

        impl DebouncerStateful<$T, $M> {
            /// Update the state.
            pub fn update(&mut self, pressed: bool) -> Option<Edge> {
                self.debouncer.update(pressed).and_then(|edge| {
                    if edge != self.last_edge {
                        self.last_edge = edge;
                        Some(edge)
                    } else {
                        None
                    }
                })
            }

            /// Return `true` if the debounced state is logical high.
            pub fn is_high(&self) -> bool {
                self.debouncer.is_high()
            }

            /// Return `true` if the debounced state is logical low.
            pub fn is_low(&self) -> bool {
                self.debouncer.is_low()
            }
        }
    };
}

impl_logic!(u8,  2,  Repeat2,  debounce_2,  debounce_stateful_2,  0b0000_0011);
impl_logic!(u8,  3,  Repeat3,  debounce_3,  debounce_stateful_3,  0b0000_0111);
impl_logic!(u8,  4,  Repeat4,  debounce_4,  debounce_stateful_4,  0b0000_1111);
impl_logic!(u8,  5,  Repeat5,  debounce_5,  debounce_stateful_5,  0b0001_1111);
impl_logic!(u8,  6,  Repeat6,  debounce_6,  debounce_stateful_6,  0b0011_1111);
impl_logic!(u8,  7,  Repeat7,  debounce_7,  debounce_stateful_7,  0b0111_1111);
impl_logic!(u8,  8,  Repeat8,  debounce_8,  debounce_stateful_8,  0b1111_1111);
impl_logic!(u16, 9,  Repeat9,  debounce_9,  debounce_stateful_9,  0b0000_0001_1111_1111);
impl_logic!(u16, 10, Repeat10, debounce_10, debounce_stateful_10, 0b0000_0011_1111_1111);
impl_logic!(u16, 11, Repeat11, debounce_11, debounce_stateful_11, 0b0000_0111_1111_1111);
impl_logic!(u16, 12, Repeat12, debounce_12, debounce_stateful_12, 0b0000_1111_1111_1111);
impl_logic!(u16, 13, Repeat13, debounce_13, debounce_stateful_13, 0b0001_1111_1111_1111);
impl_logic!(u16, 14, Repeat14, debounce_14, debounce_stateful_14, 0b0011_1111_1111_1111);
impl_logic!(u16, 15, Repeat15, debounce_15, debounce_stateful_15, 0b0111_1111_1111_1111);
impl_logic!(u16, 16, Repeat16, debounce_16, debounce_stateful_16, 0b1111_1111_1111_1111);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rising_edge() {
        // Initially not pressed
        let mut debouncer: Debouncer<u8, Repeat3> = debounce_3(false);
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
        let mut debouncer: Debouncer<u8, Repeat3> = debounce_3(false);
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
        let mut debouncer: Debouncer<u16, Repeat16> = debounce_16(false);
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
        let mut debouncer: Debouncer<u8, Repeat8> = debounce_8(false);
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
        // Regular debouncers
        assert_eq!(std::mem::size_of_val(&debounce_2(false)), 1);
        assert_eq!(std::mem::size_of_val(&debounce_8(false)), 1);
        assert_eq!(std::mem::size_of_val(&debounce_9(false)), 2);
        assert_eq!(std::mem::size_of_val(&debounce_16(false)), 2);

        // Stateful debouncers
        assert_eq!(std::mem::size_of_val(&debounce_stateful_8(false)), 2);
        assert_eq!(std::mem::size_of_val(&debounce_stateful_9(false)), 4);
    }

    /// Ensure that the initial state can be specified.
    #[test]
    fn test_initial_state() {
        let mut debouncer = debounce_2(false);
        assert_eq!(debouncer.update(false), None);
        assert_eq!(debouncer.update(false), None);
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(true), Some(Edge::Rising));

        let mut debouncer = debounce_2(false);
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(true), Some(Edge::Rising));
        assert_eq!(debouncer.update(false), None);
        assert_eq!(debouncer.update(false), Some(Edge::Falling));

        let mut debouncer = debounce_2(true);
        assert_eq!(debouncer.update(false), None);
        assert_eq!(debouncer.update(false), Some(Edge::Falling));
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(true), Some(Edge::Rising));

        let mut debouncer = debounce_2(true);
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(false), None);
        assert_eq!(debouncer.update(false), Some(Edge::Falling));

        // Stateful debouncers
        let mut debouncer = debounce_stateful_2(false);
        assert_eq!(debouncer.update(false), None);
        assert_eq!(debouncer.update(false), None);
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(true), Some(Edge::Rising));

        let mut debouncer = debounce_stateful_2(false);
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(true), Some(Edge::Rising));
        assert_eq!(debouncer.update(false), None);
        assert_eq!(debouncer.update(false), Some(Edge::Falling));

        let mut debouncer = debounce_stateful_2(true);
        assert_eq!(debouncer.update(false), None);
        assert_eq!(debouncer.update(false), Some(Edge::Falling));
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(true), Some(Edge::Rising));

        let mut debouncer = debounce_stateful_2(true);
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(true), None);
        assert_eq!(debouncer.update(false), None);
        assert_eq!(debouncer.update(false), Some(Edge::Falling));

    }
}
