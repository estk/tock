//! Component for imix board LEDs.
//!
//! This provides one Component, LedComponent, which implements
//! a userspace syscall interface to the two imix on-board LEDs.
//!
//! Usage
//! -----
//! ```rust
//! let led = LedComponent::new().finalize();
//! ```

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 6/20/2018

#![allow(dead_code)] // Components are intended to be conditionally included

// use core::marker::PhantomData;

use capsules::led;
use kernel::component::Component;
use kernel::hil::gpio;
use kernel::static_init;

// pub struct LedComponent<'a, T: 'a + gpio::Pin> {
//     led_pins: &'a T,
// }

// impl<'a, T:'a + gpio::Pin> LedComponent<'a, T> {
//     pub fn new(led_pins: &'a T) -> LedComponent<'a, T> {
//         LedComponent {
//             led_pins,
//         }
//     }

//     // unsafe fn finalize(&self) -> Option<&T> {
//     //     let x: Option<&T> = None;
//     //     // static_init!(
//     //     //     led::LED<'static, T>,
//     //     //     led::LED::new(self.led_pins)
//     //     // )
//     // }

//     // unsafe fn finalize(&self) -> Option<led::LED<'static, T>> {
//     unsafe fn finalize(&self) -> &'static led::LED<'static, T> {
//         // // let x: Option<led::LED<'static, T>> = None;
//         // static_init!(
//         //     led::LED<'static, T>,
//         //     led::LED::new(self.led_pins)
//         // )
//         // // x

//         use core::{mem, ptr};
//         // Statically allocate a read-write buffer for the value, write our
//         // initial value into it (without dropping the initial zeros) and
//         // return a reference to it.
//         let BUF: Option<led::LED<'static, T>> = None;
//         let tmp : &'static mut led::LED<'static, T> = mem::transmute(&mut BUF);
//         ptr::write(tmp as *mut led::LED<'static, T>, led::LED::new(self.led_pins));
//         tmp
//         //
//         // BUF
//     }
// }





// pub struct LedComponent<'a, T: 'a + gpio::Pin> {
//     led_pins: &'a T,
// }

// impl<'a, T:'a + gpio::Pin> LedComponent<'a, T> {
//     pub fn new(led_pins: &'a T) -> LedComponent<'a, T> {
//         LedComponent {
//             led_pins,
//         }
//     }

//     // unsafe fn finalize(&self) -> Option<&T> {
//     //     let x: Option<&T> = None;
//     //     // static_init!(
//     //     //     led::LED<'static, T>,
//     //     //     led::LED::new(self.led_pins)
//     //     // )
//     // }

//     unsafe fn finalize(&self) -> Option<led::LED<'static, T>> {
//         let x: Option<led::LED<'static, T>> = None;
//         // static_init!(
//         //     led::LED<'static, T>,
//         //     led::LED::new(self.led_pins)
//         // )
//         x
//     }
// }

#[macro_export]
macro_rules! led_component_helper {
    ($T:ty) => {

        {

            static mut BUF: Option<capsules::led::LED<'static, $T>> = None;
            &mut BUF
        };
    }
}



pub struct LedComponent<T: 'static + gpio::Pin + gpio::PinCtl> {
    led_pins: &'static [(&'static T, led::ActivationMode)],
}

impl<T: 'static + gpio::Pin + gpio::PinCtl> LedComponent<T> {
    pub fn new(led_pins: &'static [(&'static T, led::ActivationMode)]) -> LedComponent<T> {
        LedComponent {
            led_pins,
        }
    }

    pub unsafe fn finalize(&mut self, BUF: &mut Option<led::LED<'static, T>>) -> &'static led::LED<'static, T> {
        // static_init!(
        //     led::LED<'static, T>,
        //     led::LED::new(self.led_pins)
        // )

        use core::{mem, ptr};
        // Statically allocate a read-write buffer for the value, write our
        // initial value into it (without dropping the initial zeros) and
        // return a reference to it.
        // static mut BUF: Option<led::LED<'static, T>> = None;
        let tmp : &'static mut led::LED<'static, T> = mem::transmute(BUF);
        ptr::write(tmp as *mut led::LED<'static, T>, led::LED::new(self.led_pins));
        tmp
        //
        // BUF
    }
}

// pub struct LedComponent<T: 'static + gpio::Pin + gpio::PinCtl> {
//     led_pins: &'static [(&'static T, led::ActivationMode)],
// }

// impl<T: 'static + gpio::Pin + gpio::PinCtl> LedComponent<T> {
//     pub fn new(led_pins: &'static [(&'static T, led::ActivationMode)]) -> LedComponent<T> {
//         LedComponent {
//             led_pins,
//         }
//     }

//     unsafe fn finalize(&mut self) -> &'static led::LED<'static, T> {
//         static_init!(
//             led::LED<'static, T>,
//             led::LED::new(self.led_pins)
//         )
//     }
// }

// impl<T: 'static + gpio::Pin + gpio::PinCtl> LedComponent<T> {
//     pub fn new(led_pins: &'static [(&'static T, led::ActivationMode)]) -> LedComponent<T> {
//         LedComponent {
//             led_pins,
//         }
//     }

//     // unsafe fn finalize<T: 'static + gpio::Pin + gpio::PinCtl>(&mut self) -> &'static led::LED<'static, T> {
//     unsafe fn finalize<T2: 'static + gpio::Pin + gpio::PinCtl>(&mut self) -> &'static mut led::LED<'static, T> {
//         // static_init!(
//         //     led::LED<'static, T2>,
//         //     led::LED::new(self.led_pins)
//         // )

//         use core::{mem, ptr};
//         // Statically allocate a read-write buffer for the value, write our
//         // initial value into it (without dropping the initial zeros) and
//         // return a reference to it.
//         static mut BUF: Option<led::LED<'static, T2>> = None;
//         let tmp : &'static mut led::LED<'static, T2> = mem::transmute(&mut BUF);
//         ptr::write(tmp as *mut led::LED<'static, T2>, led::LED::new(self.led_pins));
//         tmp
//     }
// }

// impl<T: 'static + gpio::Pin + gpio::PinCtl> Component for LedComponent<T> {
//     type Output = &'static led::LED<'static, T>;

//     unsafe fn finalize(&mut self) -> Self::Output {
//         static_init!(
//             led::LED<'static, T>,
//             led::LED::new(self.led_pins)
//         )
//     }
// }

// pub struct LedComponent<T: 'static + gpio::Pin + gpio::PinCtl, L> {
//     led_pins: &'static [(&'static T, led::ActivationMode)],
//     phantom: PhantomData<T>,
// }

// impl<T: 'static + gpio::Pin + gpio::PinCtl, L> LedComponent<T, L> {
//     pub fn new(led_pins: &'static [(&'static T, led::ActivationMode)]) -> LedComponent<T, L> {
//         LedComponent {
//             led_pins,
//             phantom: PhantomData,
//         }
//     }

//     fn um<T: 'static + gpio::Pin + gpio::PinCtl, L, R>(&self) -> R {
//         // static_init!(
//         //     led::LED<'static, T>,
//         //     led::LED::new(self.led_pins)
//         // ) as R

//         // static_init!(
//         //     L,
//         //     led::LED::new(self.led_pins)
//         // ) as R

//         // use core::{mem, ptr};
//         // // Statically allocate a read-write buffer for the value, write our
//         // // initial value into it (without dropping the initial zeros) and
//         // // return a reference to it.
//         // type arg<P: 'static + gpio::Pin + gpio::PinCtl> = Option<led::LED<'static, P>>;
//         // static mut BUF: arg<P> = None;
//         // let tmp : &'static mut led::LED<'static, P> = mem::transmute(&mut BUF);
//         // ptr::write(tmp as *mut led::LED<'static, P>, led::LED::new(self.led_pins));
//         // tmp

//         use core::{mem, ptr};
//         // Statically allocate a read-write buffer for the value, write our
//         // initial value into it (without dropping the initial zeros) and
//         // return a reference to it.
//         type arg = Option<L>;
//         static mut BUF: arg = None;
//         let tmp : &'static mut L = mem::transmute(&mut BUF);
//         ptr::write(tmp as *mut L, led::LED::new(self.led_pins));
//         tmp


//     }
// }

// impl<T: 'static + gpio::Pin + gpio::PinCtl, L> Component for LedComponent<T, L> {
//     type Output = &'static led::LED<'static, T>;

//     unsafe fn finalize(&mut self) -> Self::Output {
//         // static_init!(
//         //     L,
//         //     led::LED::new(self.led_pins)
//         // )
//         //

//         // static_init!(
//         //     led::LED<'static, T>,
//         //     led::LED::new(self.led_pins)
//         // )

//         self.um::<L, Self::Output>()


//         // use core::{mem, ptr};
//         // // Statically allocate a read-write buffer for the value, write our
//         // // initial value into it (without dropping the initial zeros) and
//         // // return a reference to it.
//         // static mut BUF: Option<led::LED<'static, T>> = None;
//         // let tmp : &'static mut led::LED<'static, T> = mem::transmute(&mut BUF);
//         // ptr::write(tmp as *mut led::LED<'static, T>, led::LED::new(self.led_pins));
//         // tmp
//     }
// }
